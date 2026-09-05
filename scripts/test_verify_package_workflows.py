"""Guard against unsupported or stale package evidence acquiring usable badges."""
import json
from pathlib import Path
import tempfile
import unittest

import verify_package_workflows as verifier


class VerificationEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.audit = json.loads((verifier.DIRECTORY / 'popularity-audit.json').read_text())
        self.report = json.loads((verifier.DIRECTORY / 'verified-packages.json').read_text())

    def verified(self):
        return next(r for r in self.report['records'] if r['status'] == 'verified')

    def reject(self):
        with self.assertRaises(ValueError):
            verifier.validate_report(self.report, self.audit)

    def test_complete_checked_in_evidence_and_reviewed_hooks(self):
        verifier.validate_report(self.report, self.audit)
        self.assertEqual(len(self.report['records']), 44)

    def test_stale_source_and_reviewed_case_digests_are_rejected(self):
        for field in ('source_report_sha256', 'cases_sha256'):
            with self.subTest(field=field):
                saved = self.report[field]
                self.report[field] = '0' * 64
                self.reject()
                self.report[field] = saved

    def test_verified_requires_detached_execution_and_completion_assertion(self):
        record = self.verified()
        for field, value in (('returncode', 1), ('stdout', ''), ('output_truncated', True)):
            with self.subTest(field=field):
                step = next(s for s in record['evidence']['steps'] if s['name'] == 'detached-standalone-workflow')
                saved = step[field]
                step[field] = value
                self.reject()
                step[field] = saved

    def test_limited_cannot_promote_an_installation_failure(self):
        record = next(r for r in self.report['records'] if r['status'] == 'unsupported' and r['kind'] == 'library')
        record['status'] = record['platforms'][0]['status'] = 'limited'
        self.reject()

    def test_claim_cannot_expand_beyond_the_reviewed_scope(self):
        self.verified()['scope'].append('All Python package features work.')
        self.reject()

    def test_other_platform_cannot_inherit_badge(self):
        self.verified()['platforms'][0]['target'] = 'linux-x86_64'
        self.reject()

    def test_dependency_lock_and_artifact_changes_are_rejected(self):
        record = self.verified()
        record['evidence']['lock']['packages'][0]['sha256'] = '0' * 64
        self.reject()

    def test_missing_candidate_is_not_a_complete_verification(self):
        self.report['records'].pop()
        self.reject()

    def test_binary_evidence_cannot_be_relabelled_as_a_new_release(self):
        self.report['release'] = '99.0.0'
        self.reject()

    def test_dependency_bearing_approval_cannot_enter_root_only_catalog(self):
        proof = self.verified()['evidence']
        proof['lock']['packages'].append({'name': 'untested-dependency'})
        proof['lock_json_sha256'] = verifier.json_digest(proof['lock'])
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'catalog.json'
            original = (verifier.DIRECTORY / 'catalog.json').read_bytes()
            output.write_bytes(original)
            with self.assertRaisesRegex(ValueError, 'lock-bound'):
                verifier.promote(self.report, self.audit, output)
            self.assertEqual(output.read_bytes(), original)

    def test_network_failures_are_not_compatibility_failures(self):
        for message in ('network request timed out', 'HTTP status 503', 'connection reset'):
            self.assertEqual(verifier.installation_status(message), 'untested')
        self.assertEqual(verifier.installation_status('environment markers are not supported'), 'unsupported')

    def test_readable_reason_preserves_the_missing_api(self):
        self.assertEqual(verifier.friendly_reason("ImportError: No module named 'atexit'"),
                         'Needs the atexit module, which this runtime does not provide yet.')


if __name__ == '__main__':
    unittest.main()
