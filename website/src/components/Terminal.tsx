"use client";

import { useEffect, useState } from "react";

// ANSI color codes to CSS classes
const ansiColors: Record<string, string> = {
  "30": "text-gray-900 dark:text-gray-300",
  "31": "text-red-500",
  "32": "text-green-500",
  "33": "text-yellow-500",
  "34": "text-blue-500",
  "35": "text-purple-500",
  "36": "text-cyan-500",
  "37": "text-gray-100",
  "90": "text-gray-500",
  "91": "text-red-400",
  "92": "text-green-400",
  "93": "text-yellow-400",
  "94": "text-blue-400",
  "95": "text-purple-400",
  "96": "text-cyan-400",
  "97": "text-white",
};

const ansiBgColors: Record<string, string> = {
  "40": "bg-gray-900",
  "41": "bg-red-500",
  "42": "bg-green-500",
  "43": "bg-yellow-500",
  "44": "bg-blue-500",
  "45": "bg-purple-500",
  "46": "bg-cyan-500",
  "47": "bg-gray-100",
};

interface ParsedSegment {
  start: number;
  text: string;
  classes: string[];
}

function parseAnsi(text: string): ParsedSegment[] {
  const segments: ParsedSegment[] = [];
  const controlSequenceIntroducer = String.fromCharCode(27) + String.raw`\[`;
  const regex = new RegExp(`${controlSequenceIntroducer}([0-9;]*)m`, "g");
  let lastIndex = 0;
  let currentClasses: string[] = [];

  while (true) {
    const match = regex.exec(text);
    if (match === null) break;

    // Add text before this escape sequence
    if (match.index > lastIndex) {
      segments.push({
        start: lastIndex,
        text: text.slice(lastIndex, match.index),
        classes: [...currentClasses],
      });
    }

    // Parse the escape codes
    const codes = match[1].split(";").filter(Boolean);
    for (const code of codes) {
      if (code === "0") {
        currentClasses = [];
      } else if (code === "1") {
        currentClasses.push("font-bold");
      } else if (code === "2") {
        currentClasses.push("opacity-60");
      } else if (code === "3") {
        currentClasses.push("italic");
      } else if (code === "4") {
        currentClasses.push("underline");
      } else if (code === "9") {
        currentClasses.push("line-through");
      } else if (ansiColors[code]) {
        // Remove previous text color and add new one
        currentClasses = currentClasses.filter((c) => !c.startsWith("text-"));
        currentClasses.push(ansiColors[code]);
      } else if (ansiBgColors[code]) {
        currentClasses = currentClasses.filter((c) => !c.startsWith("bg-"));
        currentClasses.push(ansiBgColors[code]);
      }
    }

    lastIndex = regex.lastIndex;
  }

  // Add remaining text
  if (lastIndex < text.length) {
    segments.push({
      start: lastIndex,
      text: text.slice(lastIndex),
      classes: [...currentClasses],
    });
  }

  return segments;
}

interface TerminalProps {
  children: string;
  title?: string;
  animated?: boolean;
  typingSpeed?: number;
  className?: string;
}

export function Terminal({
  children,
  title = "Terminal",
  animated = false,
  typingSpeed = 30,
  className = "",
}: TerminalProps) {
  const [displayedText, setDisplayedText] = useState(animated ? "" : children);

  useEffect(() => {
    if (!animated) {
      setDisplayedText(children);
      return;
    }

    setDisplayedText("");
    let index = 0;
    const interval = setInterval(() => {
      if (index < children.length) {
        setDisplayedText(children.slice(0, index + 1));
        index++;
      } else {
        clearInterval(interval);
      }
    }, typingSpeed);

    return () => clearInterval(interval);
  }, [children, animated, typingSpeed]);

  const segments = parseAnsi(displayedText);

  return (
    <div
      className={`rounded-lg overflow-hidden border border-gray-200 dark:border-gray-800 shadow-lg my-6 ${className}`}
    >
      {/* Terminal header */}
      <div className="bg-gray-100 dark:bg-gray-800 px-4 py-2 flex items-center gap-2">
        <div className="flex gap-1.5">
          <div className="w-3 h-3 rounded-full bg-red-500" />
          <div className="w-3 h-3 rounded-full bg-yellow-500" />
          <div className="w-3 h-3 rounded-full bg-green-500" />
        </div>
        <span className="text-sm text-gray-600 dark:text-gray-400 ml-2">
          {title}
        </span>
      </div>
      {/* Terminal body */}
      <div className="bg-gray-950 p-4 font-mono text-sm leading-relaxed overflow-x-auto">
        <pre className="text-gray-100 whitespace-pre">
          {segments.map((segment) => (
            <span key={segment.start} className={segment.classes.join(" ")}>
              {segment.text}
            </span>
          ))}
          {animated && <span className="animate-pulse">_</span>}
        </pre>
      </div>
    </div>
  );
}

// Pre-styled terminal output examples
export function TerminalDemo() {
  const demoOutput = `\x1b[36m$\x1b[0m kipferl dev demo.py
\x1b[1;32mKipferl dev\x1b[0m \x1b[2mwatching ./\x1b[0m
\x1b[36m→\x1b[0m Running \x1b[1mdemo.py\x1b[0m

\x1b[1;34m╭─ Release ─────────────────────────────╮\x1b[0m
\x1b[1;34m│\x1b[0m Deploying build...                    \x1b[1;34m│\x1b[0m
\x1b[1;34m╰───────────────────────────────────────╯\x1b[0m

\x1b[1;32m✓\x1b[0m Built commit a1b2c3d

\x1b[90m┌───────────┬───────┬──────┐\x1b[0m
\x1b[90m│\x1b[0m Artifact  \x1b[90m│\x1b[0m Size  \x1b[90m│\x1b[0m Time \x1b[90m│\x1b[0m
\x1b[90m├───────────┼───────┼──────┤\x1b[0m
\x1b[90m│\x1b[0m app-linux \x1b[90m│\x1b[0m \x1b[32m900KB\x1b[0m \x1b[90m│\x1b[0m \x1b[33m6ms\x1b[0m  \x1b[90m│\x1b[0m
\x1b[90m│\x1b[0m app-macos \x1b[90m│\x1b[0m \x1b[32m910KB\x1b[0m \x1b[90m│\x1b[0m \x1b[33m7ms\x1b[0m  \x1b[90m│\x1b[0m
\x1b[90m└───────────┴───────┴──────┘\x1b[0m`;

  return <Terminal title="demo.py">{demoOutput}</Terminal>;
}
