"use client";

import {
  ArrowDown,
  ArrowRight,
  ArrowUpRight,
  Check,
  CheckCheck,
  ChevronRight,
  Code2,
  Copy,
  FileCode2,
  FolderOpen,
  Github,
  Layers3,
  Package,
  Play,
  ShieldCheck,
  Terminal,
  Zap,
} from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import styles from "./home.module.css";

const installCommand = "brew install niklas-heer/tap/kipferl";
const chapters = [
  {
    title: "Create a project",
    detail: "A working app. Tests included.",
    time: 0,
    icon: FolderOpen,
  },
  {
    title: "Check your packages",
    detail: "Real evidence. Locked dependencies.",
    time: 15,
    icon: ShieldCheck,
  },
  {
    title: "Ship one executable",
    detail: "Code, runtime, and resources together.",
    time: 30,
    icon: Package,
  },
];
const starters = [
  {
    id: "cli",
    label: "Command-line tool",
    description:
      "Arguments, helpful output, and a starter test. Your next useful command starts here.",
  },
  {
    id: "api",
    label: "API client",
    description:
      "Fetch JSON with built-in HTTP support. Turn an API into a tool your team can use.",
  },
  {
    id: "interactive",
    label: "Interactive app",
    description:
      "Give your script a friendly interface with keyboard-driven prompts and selections.",
  },
];

function InstallCommand() {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");
  useEffect(() => {
    if (status === "idle") return;
    const timer = setTimeout(() => setStatus("idle"), 2500);
    return () => clearTimeout(timer);
  }, [status]);
  async function copy() {
    try {
      await navigator.clipboard.writeText(installCommand);
      setStatus("copied");
    } catch {
      setStatus("failed");
    }
  }
  return (
    <div>
      <div className={styles.install}>
        <span aria-hidden="true">$</span>
        <code>{installCommand}</code>
        <button
          type="button"
          onClick={copy}
          aria-label={
            status === "copied"
              ? "Install command copied"
              : "Copy install command"
          }
        >
          {status === "copied" ? <Check size={16} /> : <Copy size={16} />}
        </button>
      </div>
      <output className={styles.copyStatus}>
        {status === "copied"
          ? "Copied. See you in the terminal."
          : status === "failed"
            ? "Select the command above to copy it."
            : "macOS & Linux · ARM64 & x86_64"}
      </output>
    </div>
  );
}

function WorkflowDemo() {
  const video = useRef<HTMLVideoElement>(null);
  const pendingSeek = useRef<number | null>(null);
  const [active, setActive] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [error, setError] = useState(false);
  function playChapter(index: number) {
    const player = video.current;
    if (!player) return;
    setActive(index);
    const chapter = chapters[index];
    if (!chapter) return;
    if (player.readyState >= 1) player.currentTime = chapter.time;
    else pendingSeek.current = chapter.time;
    void player.play().catch(() => setPlaying(false));
    player.scrollIntoView({
      block: "start",
      behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches
        ? "auto"
        : "smooth",
    });
  }
  return (
    <section
      id="demo"
      className={styles.demoSection}
      aria-labelledby="demo-title"
    >
      <div className={styles.demoHeading}>
        <div>
          <span className={styles.eyebrow}>LESS SETUP. MORE SHIPPING.</span>
          <h2 id="demo-title">Meet your new workflow.</h2>
        </div>
        <span className={styles.recordingLabel}>
          <span /> 51 sec · Kipferl 0.7.1
        </span>
      </div>
      <div className={styles.terminalWindow}>
        <div className={styles.terminalBar}>
          <span className={styles.windowDots} aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
          <span>
            <Terminal size={14} /> hello / kipferl
          </span>
          <span className={styles.terminalTag}>
            REAL TERMINAL. REAL OUTPUT.
          </span>
        </div>
        <div className={styles.videoFrame}>
          <video
            ref={video}
            controls={playing}
            playsInline
            muted
            preload="none"
            poster="/demos/kipferl-0.7.1.webp"
            aria-label="Silent terminal recording: create a Kipferl project, install and check tzdata, and build a standalone application. A text walkthrough follows."
            onPlay={() => setPlaying(true)}
            onError={() => setError(true)}
            onEnded={() => setPlaying(false)}
            onLoadedMetadata={() => {
              if (pendingSeek.current !== null && video.current) {
                video.current.currentTime = pendingSeek.current;
                pendingSeek.current = null;
              }
            }}
            onTimeUpdate={() => {
              const time = video.current?.currentTime ?? 0;
              const index = chapters.findLastIndex(
                (chapter) => time >= chapter.time,
              );
              if (index >= 0) setActive(index);
            }}
          >
            <source src="/demos/kipferl-0.7.1.mp4" type="video/mp4" />
            Your browser does not support this video. Read the walkthrough
            below.
          </video>
          {!playing && !error && (
            <button
              className={styles.playOverlay}
              type="button"
              onClick={() => playChapter(0)}
            >
              <span>
                <Play size={24} fill="currentColor" />
              </span>
              <strong>Watch it come together</strong>
              <small>Create → check → ship</small>
            </button>
          )}
          {error && (
            <p className={styles.videoError}>
              The recording could not load.{" "}
              <a href="/demos/kipferl-0.7.1.mp4">Download the video</a> or
              follow the text walkthrough below.
            </p>
          )}
        </div>
        <div className={styles.chapters}>
          {chapters.map((chapter, index) => (
            <button
              key={chapter.title}
              type="button"
              className={active === index ? styles.activeChapter : ""}
              aria-pressed={active === index}
              onClick={() => playChapter(index)}
            >
              <span className={styles.chapterNumber}>0{index + 1}</span>
              <span>
                <strong>{chapter.title}</strong>
                <small>{chapter.detail}</small>
              </span>
              <chapter.icon size={19} aria-hidden="true" />
            </button>
          ))}
        </div>
      </div>
      <div className={styles.demoFootnote}>
        <details>
          <summary>
            Prefer reading? Follow the workflow <ChevronRight size={14} />
          </summary>
          <div className={styles.transcript}>
            <p>
              The recording uses the released macOS Apple Silicon CLI. The same
              workflow is available on all four release targets.
            </p>
            <ol>
              <li>
                Create a CLI project, run its greeting, and execute its starter
                test.
              </li>
              <li>
                Install the reviewed tzdata 2025.2 wheel and check the installed
                files. Add a small app that reads a bundled timezone data
                resource.
              </li>
              <li>
                Build an executable, remove the original project and caches, and
                run the executable independently.
              </li>
            </ol>
            <pre>{`kipferl new hello --template cli\ncd hello\nkipferl run -- --name Ada\nkipferl test\nkipferl add 'tzdata==2025.2'\nkipferl deps check\nkipferl build`}</pre>
            <p>
              Save this as <code>zones.py</code> inside the project:
            </p>
            <pre>{`import os
import tzdata

path = os.path.join(os.path.dirname(tzdata.__file__), "zoneinfo/UTC")
with open(path, "rb") as zone:
    assert zone.read(4) == b"TZif"
print("tzdata " + tzdata.__version__ + ": UTC data bundled")`}</pre>
            <pre>{`kipferl build zones.py --mode universal -o dist/zones
./dist/zones
# tzdata 2025.2: UTC data bundled`}</pre>
            <p>
              The recording takes this one step further: it runs the executable
              after deleting its temporary source project, installed packages,
              and caches.
            </p>
            <p>
              The tzdata checks cover version constants and resource headers,
              not timezone calculations.{" "}
              <Link href="/docs/guides/packages">
                Try the complete package example →
              </Link>
            </p>
          </div>
        </details>
        <a
          href="https://github.com/niklas-heer/kipferl/blob/main/demo.tape"
          className={styles.textLink}
        >
          Made with VHS <ArrowUpRight size={14} />
        </a>
      </div>
    </section>
  );
}

function StarterPicker() {
  const [selected, setSelected] = useState("cli");
  const starter = starters.find((item) => item.id === selected) ?? starters[0];
  return (
    <div className={styles.starterCard}>
      <div className={styles.cardIntro}>
        <span className={styles.cardIcon}>
          <FolderOpen size={21} />
        </span>
        <span className={styles.eyebrow}>01 / START WITH SOMETHING REAL</span>
      </div>
      <h3>
        An empty folder.
        <br />A running app.
      </h3>
      <fieldset className={styles.starterTabs}>
        <legend className="sr-only">Choose a project template</legend>
        {starters.map((item) => (
          <button
            key={item.id}
            type="button"
            aria-pressed={selected === item.id}
            onClick={() => setSelected(item.id)}
          >
            {item.label}
          </button>
        ))}
      </fieldset>
      <p aria-live="polite">{starter?.description}</p>
      <div className={styles.codeBlock}>
        <span className={styles.codeComment}>
          # Your project, ready to work on
        </span>
        <code>kipferl new hello --template {selected}</code>
        <code>cd hello && kipferl run</code>
      </div>
      <div className={styles.fileTree}>
        <span>
          <FileCode2 size={15} /> hello.py
        </span>
        <span>
          <CheckCheck size={15} /> tests/
        </span>
        <span>
          <Code2 size={15} /> editor stubs
        </span>
        <span>
          <FolderOpen size={15} /> kipferl.json
        </span>
      </div>
      <Link
        className={styles.textLink}
        href="/docs/getting-started/quick-start"
      >
        Build your first app <ArrowUpRight size={16} />
      </Link>
    </div>
  );
}

export default function HomePage() {
  return (
    <main className={styles.home}>
      <div className={styles.shell}>
        <section className={styles.hero}>
          <div className={styles.heroTopline}>
            <span className={styles.eyebrow}>THE PYTHON-STYLE CLI TOOLKIT</span>
            <Link href="/blog/kipferl-0-7" className={styles.releaseBadge}>
              <span /> 0.7 is here <ArrowUpRight size={14} />
            </Link>
          </div>
          <div className={styles.heroGrid}>
            <div>
              <h1>
                From Python script
                <br />
                to <span>shipped tool.</span>
              </h1>
              <p className={styles.heroLead}>
                Write familiar code. Add compatible packages.
                <br className={styles.desktopBreak} /> Hand someone a single
                executable.
              </p>
            </div>
            <div className={styles.heroAction}>
              <p>
                A complete workflow for small tools
                <br />
                with a lot to do.
              </p>
              <Link
                className={styles.primaryButton}
                href="/docs/getting-started/quick-start"
              >
                Build your first CLI <ArrowRight size={18} />
              </Link>
              <a className={styles.secondaryButton} href="#demo">
                See it in action <ArrowDown size={16} />
              </a>
            </div>
          </div>
          <div className={styles.heroBottom}>
            <InstallCommand />
            <div className={styles.heroPromise}>
              <span>
                <Check size={15} /> Built-in terminal UI
              </span>
              <span>
                <Check size={15} /> No Python on the target machine
              </span>
            </div>
          </div>
        </section>

        <WorkflowDemo />

        <section
          className={styles.proofStrip}
          aria-label="Release verification"
        >
          <div>
            <strong>0.7.2</strong>
            <span>Stable & ready to build with</span>
          </div>
          <div>
            <strong>4 targets</strong>
            <span>macOS + Linux · ARM64 + x86_64</span>
          </div>
          <div>
            <strong>1,725 checks</strong>
            <span>Available compatibility checks passed</span>
          </div>
          <Link href="https://github.com/niklas-heer/kipferl/releases/tag/v0.7.2">
            Inspect the release evidence <ArrowUpRight size={17} />
          </Link>
        </section>

        <section className={styles.features} aria-labelledby="features-title">
          <div className={styles.sectionHeading}>
            <div>
              <span className={styles.eyebrow}>
                A LITTLE TOOL. A COMPLETE WORKFLOW.
              </span>
              <h2 id="features-title">
                Everything between
                <br />
                an idea and <span>“here, try this.”</span>
              </h2>
            </div>
            <p>
              Project scaffolding, checked dependencies, and portable builds.
              All part of the same CLI.
            </p>
          </div>
          <div className={styles.featureGrid}>
            <StarterPicker />
            <div className={styles.packageCard}>
              <div className={styles.cardIntro}>
                <span className={styles.cardIcon}>
                  <ShieldCheck size={21} />
                </span>
                <span className={styles.eyebrow}>
                  02 / KNOW WHAT YOU ARE ADDING
                </span>
              </div>
              <h3>
                Packages with
                <br />
                receipts.
              </h3>
              <p>
                Check PyPI dependencies against your exact runtime. Lock their
                versions and hashes. Restore them offline once their wheels are
                cached.
              </p>
              <div className={styles.packageReceipt}>
                <div className={styles.receiptTitle}>
                  <Package size={18} />
                  <strong>tzdata</strong>
                  <span>2025.2</span>
                </div>
                <div className={styles.receiptStatus}>
                  <span>
                    <CheckCheck size={15} /> Tested
                  </span>
                  <code>pure Python wheel</code>
                </div>
                <ul>
                  <li>
                    <Check size={14} /> Wheel checksum verified
                  </li>
                  <li>
                    <Check size={14} /> Source compilation passed
                  </li>
                  <li>
                    <Check size={14} /> Reviewed resource checks passed
                  </li>
                </ul>
                <div className={styles.receiptScope}>
                  Exact artifact. Exact runtime. A stated test scope.
                </div>
              </div>
              <code className={styles.singleCommand}>
                $ kipferl add 'tzdata==2025.2'
              </code>
              <Link className={styles.textLink} href="/docs/guides/packages">
                Explore the package workflow <ArrowUpRight size={16} />
              </Link>
            </div>
            <div className={styles.shipCard}>
              <div className={styles.shipCopy}>
                <div className={styles.cardIntro}>
                  <span className={styles.cardIcon}>
                    <Layers3 size={21} />
                  </span>
                  <span className={styles.eyebrow}>
                    03 / GIVE YOUR TOOL A LIFE OUTSIDE YOUR LAPTOP
                  </span>
                </div>
                <h3>
                  Your app.
                  <br />
                  All packed.
                </h3>
                <p>
                  Bundle local modules, package resources, and selected assets
                  with the runtime. The person running your tool needs neither
                  Python nor Kipferl installed.
                </p>
                <Link className={styles.textLink} href="/docs/guides/packaging">
                  See how packaging works <ArrowUpRight size={16} />
                </Link>
              </div>
              <div
                className={styles.bundleDiagram}
                role="img"
                aria-label="Python code, resources, and the runtime combine into one executable for a selected target"
              >
                <div className={styles.bundleInputs}>
                  <span>
                    <FileCode2 size={17} /> Your Python
                  </span>
                  <span>
                    <Package size={17} /> Packages + data
                  </span>
                  <span>
                    <Zap size={17} /> Runtime
                  </span>
                </div>
                <div className={styles.bundleArrow}>
                  <ArrowRight size={26} />
                </div>
                <div className={styles.bundleOutput}>
                  <Terminal size={32} />
                  <strong>hello</strong>
                  <span>One executable</span>
                </div>
                <code>kipferl build</code>
                <p>One binary per operating system and CPU target.</p>
              </div>
            </div>
          </div>
        </section>

        <section className={styles.nativeSection}>
          <div>
            <span className={styles.eyebrow}>
              THE GOOD STUFF IS ALREADY BUILT IN
            </span>
            <h2>
              Make the terminal
              <br />a nicer place to be.
            </h2>
            <p>
              Tables, boxes, prompts, and progress. HTTP, SQLite, and everyday
              file formats. Familiar dotted imports connect it all.
            </p>
            <Link className={styles.textLink} href="/docs/modules">
              Explore the runtime modules <ArrowUpRight size={16} />
            </Link>
          </div>
          <div className={styles.nativeCode}>
            <div>
              <span>app.py</span>
              <Code2 size={16} />
            </div>
            <pre>
              <code>
                <span className={styles.syntaxKeyword}>import</span>
                {" urllib.parse as urls\n"}
                <span className={styles.syntaxKeyword}>import</span>
                {" tui\n\n"}
                <span className={styles.codeComment}>
                  # Native modules, ordinary imports
                </span>
                {"\ntui.box(\n    urls.quote("}
                <span className={styles.syntaxString}>"hello world"</span>
                {"),\n    title="}
                <span className={styles.syntaxString}>"Ready to ship"</span>
                {",\n)"}
              </code>
            </pre>
            <div className={styles.nativeOutput}>
              <span>Ready to ship</span>
              <code>hello%20world</code>
            </div>
            <Link href="/docs/modules/tui">
              Meet the terminal UI toolkit <ArrowRight size={15} />
            </Link>
          </div>
        </section>

        <section className={styles.evidenceSection}>
          <div>
            <span className={styles.eyebrow}>CLEAR ABOUT WHAT WORKS</span>
            <h2>
              Less guesswork.
              <br />
              More evidence.
            </h2>
            <p>
              Kipferl is a focused Python-style runtime. It supports a useful
              subset of Python, with explicit boundaries for packages and APIs.
            </p>
          </div>
          <div className={styles.evidenceCards}>
            <Link href="/docs/guides/package-audit">
              <span className={styles.evidenceNumber}>4 ratings</span>
              <div>
                <h3>Know what works. See what’s next.</h3>
                <p>
                  Find verified workflows, read the limits, and explore 100
                  support priorities. Every rating states its evidence and
                  limits.
                </p>
              </div>
              <ArrowUpRight size={20} />
            </Link>
            <Link href="/docs/guides/packages#first-version-boundaries">
              <ShieldCheck size={27} />
              <div>
                <h3>Useful limits, stated up front.</h3>
                <p>
                  Pure Python wheels. No native extensions or source builds.
                  Tested records describe specific behavior on an exact runtime.
                </p>
              </div>
              <ArrowUpRight size={20} />
            </Link>
          </div>
        </section>

        <section className={styles.finalCta}>
          <Image src="/kipferl-logo.png" alt="" width={64} height={64} />
          <span className={styles.eyebrow}>
            SMALL TOOLS DESERVE A GOOD TOOLKIT.
          </span>
          <h2>What will you ship?</h2>
          <div>
            <Link
              className={styles.primaryButton}
              href="/docs/getting-started/quick-start"
            >
              Make something useful <ArrowRight size={18} />
            </Link>
            <Link
              className={styles.secondaryButton}
              href="https://github.com/niklas-heer/kipferl"
            >
              <Github size={18} /> Explore the source
            </Link>
          </div>
          <p>Open source. MIT licensed. Built with Rust and PocketPy.</p>
        </section>
        <footer className={styles.footer}>
          <Link href="/" className={styles.footerBrand}>
            <Image src="/kipferl-logo.png" alt="" width={24} height={24} />{" "}
            Kipferl
          </Link>
          <span>Python-style code. Standalone tools.</span>
          <nav aria-label="Footer">
            <Link href="/docs">Docs</Link>
            <Link href="/blog/kipferl-0-7">What’s new</Link>
            <Link href="/docs/guides/development">Contribute</Link>
            <Link href="https://github.com/niklas-heer/kipferl">GitHub</Link>
          </nav>
        </footer>
      </div>
    </main>
  );
}
