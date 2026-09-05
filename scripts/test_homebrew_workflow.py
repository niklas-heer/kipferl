"""Exercise Homebrew workflow shell steps against local Git and mocked PR APIs."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

WORKFLOW = Path(__file__).resolve().parents[1] / '.github/workflows/release.yml'
PR_URL = 'https://github.com/niklas-heer/homebrew-tap/pull/99'


def step_script(name):
    text = WORKFLOW.read_text().split('      - name: ' + name + '\n', 1)[1]
    block = text.split('\n      - name:', 1)[0].split('        run: |\n', 1)[1]
    return '\n'.join(line[10:] if line.startswith('          ') else line for line in block.splitlines()) + '\n'


class HomebrewWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='kipferl-tap-pr-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.seed = self.root / 'seed'
        self.seed.mkdir()
        self.git('init', '--initial-branch=main', cwd=self.seed)
        self.git('config', 'user.name', 'fixture', cwd=self.seed)
        self.git('config', 'user.email', 'fixture@example.invalid', cwd=self.seed)
        (self.seed / 'Formula').mkdir()
        (self.seed / 'Formula/kipferl.rb').write_text('previous formula\n')
        self.git('add', '.', cwd=self.seed)
        self.git('commit', '-m', 'initial formula', cwd=self.seed)
        self.remote = self.root / 'tap.git'
        self.git('clone', '--bare', str(self.seed), str(self.remote), cwd=self.root)
        self.main_head = self.git('rev-parse', 'refs/heads/main', cwd=self.remote).strip()
        hook = self.remote / 'hooks/pre-receive'
        hook.write_text('#!/bin/sh\nwhile read old new ref; do\n  if [ "$ref" = refs/heads/main ]; then exit 1; fi\ndone\n')
        hook.chmod(0o755)
        self.work = self.root / 'work'
        self.git('clone', str(self.remote), str(self.work), cwd=self.root)
        tools = self.root / 'tools'
        tools.mkdir()
        gh = tools / 'gh'
        gh.write_text(f'#!{sys.executable}\n' + r'''
import json, os
from pathlib import Path
import sys
args = sys.argv[1:]
with open(os.environ['GH_CALLS'], 'a') as log:
    log.write(json.dumps(args) + '\n')
if args[0] == 'api':
    print('main')
elif args[:2] == ['pr', 'list']:
    print(os.environ.get('EXISTING_PR', ''))
elif args[:2] == ['pr', 'create']:
    body = Path(args[args.index('--body-file') + 1]).read_text()
    assert '0.7.0' in body and 'only after this PR merges' in body
    print(os.environ['PR_URL'])
elif args[:2] == ['pr', 'merge']:
    assert '--admin' not in args
    assert '--auto' in args and '--squash' in args and '--match-head-commit' in args
    raise SystemExit(int(os.environ.get('MERGE_EXIT', '0')))
elif args[:2] == ['pr', 'view']:
    print(os.environ.get('PR_STATE', 'OPEN'))
else:
    raise SystemExit(98)
''')
        gh.chmod(0o755)
        self.summary = self.root / 'summary'
        self.calls = self.root / 'calls'
        self.env = {**os.environ, 'PATH': str(tools) + os.pathsep + os.environ['PATH'],
                    'RELEASE_VERSION': '0.7.0', 'TAP_REPOSITORY': 'niklas-heer/homebrew-tap',
                    'TAP_BRANCH': 'codex/kipferl-0.7.0', 'TAP_BASE': 'main',
                    'RUNNER_TEMP': str(self.root), 'GITHUB_STEP_SUMMARY': str(self.summary),
                    'GITHUB_OUTPUT': str(self.root / 'outputs'), 'GH_CALLS': str(self.calls),
                    'GH_TOKEN': 'fixture-token', 'PR_URL': PR_URL}

    def git(self, *args, cwd):
        result = subprocess.run(['git', *args], cwd=cwd, text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result.stdout

    def step(self, name, **env):
        result = subprocess.run(['/bin/bash', '-euo', 'pipefail', '-c', step_script(name)],
                                cwd=self.work, env={**self.env, **env}, text=True,
                                capture_output=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def publish(self, **env):
        self.step('Prepare Homebrew release branch')
        (self.work / 'Formula/kipferl.rb').write_text('verified formula 0.7.0\n')
        return self.step('Publish Homebrew pull request', **env)

    def gh_calls(self):
        return [json.loads(line) for line in self.calls.read_text().splitlines()]

    def test_branch_push_and_blocked_merge_leave_truthful_pending_pr(self):
        self.publish(MERGE_EXIT='1')
        self.assertEqual(self.git('rev-parse', 'refs/heads/main', cwd=self.remote).strip(), self.main_head)
        self.assertIn('verified formula 0.7.0', self.git('show', 'refs/heads/codex/kipferl-0.7.0:Formula/kipferl.rb', cwd=self.remote))
        summary = self.summary.read_text()
        self.assertIn('pending (OPEN)', summary)
        self.assertIn('not yet available', summary)
        self.assertIn(PR_URL, summary)
        self.assertTrue(any(call[:2] == ['pr', 'create'] for call in self.gh_calls()))

    def test_merged_state_is_required_before_reporting_promotion(self):
        self.publish(PR_STATE='MERGED')
        self.assertIn('formula v0.7.0 merged:', self.summary.read_text())
        self.assertNotIn('pending', self.summary.read_text())

    def test_rerun_reuses_remote_branch_and_open_pr_without_duplicate_commit(self):
        self.publish()
        original = self.git('rev-parse', 'refs/heads/codex/kipferl-0.7.0', cwd=self.remote)
        self.work = self.root / 'retry'
        self.git('clone', str(self.remote), str(self.work), cwd=self.root)
        self.calls.write_text('')
        self.publish(EXISTING_PR=PR_URL)
        self.assertEqual(self.git('rev-parse', 'refs/heads/codex/kipferl-0.7.0', cwd=self.remote), original)
        self.assertFalse(any(call[:2] == ['pr', 'create'] for call in self.gh_calls()))

    def test_unchanged_formula_does_not_push_or_create_pr(self):
        self.step('Prepare Homebrew release branch')
        self.step('Publish Homebrew pull request')
        self.assertIn('already current on main', self.summary.read_text())
        self.assertFalse(any(call[0] == 'pr' for call in self.gh_calls()))
        self.assertEqual(self.git('branch', '--list', 'codex/*', cwd=self.remote).strip(), '')

    def test_shell_blocks_are_valid_and_token_permissions_are_documented(self):
        for name in ('Prepare Homebrew release branch', 'Publish Homebrew pull request'):
            result = subprocess.run(['/bin/bash', '-n'], input=step_script(name), text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
        job = WORKFLOW.read_text().split('  update-homebrew:\n', 1)[1]
        self.assertIn('GH_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}', job)
        self.assertIn('Contents: write and Pull requests: write', job)
        self.assertNotIn('--admin', job)
        self.assertNotIn('git push\n', job)


if __name__ == '__main__':
    unittest.main()
