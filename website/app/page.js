"use client";

import { useEffect, useRef, useState } from "react";
import TextType from "../components/TextType";
import LiveFlightScene from "../components/LiveFlightScene";

const installCommand =
  "curl -fsSL https://flysoar-cli.vercel.app/install.sh | sh";

const commandExamples = [
  {
    label: "One-way",
    command: "flysoar search -o SFO -d JFK -D 2026-07-15",
    highlights: ["-o SFO", "-d JFK"],
  },
  {
    label: "Round-trip",
    command: "flysoar search -o SFO -d JFK -D 2026-07-15 -r 2026-07-22 -c business",
    highlights: ["-D 2026-07-15", "-r 2026-07-22"],
  },
  {
    label: "Multi-city",
    command: "flysoar search --slices SFO,JFK,2026-07-15 LHR,SFO,2026-07-22",
    highlights: ["--slices"],
  },
  {
    label: "Table",
    command: "flysoar search -o SFO -d JFK -D 2026-07-15 -O table --nonstop-only -s price",
    highlights: ["-O table"],
  },
  {
    label: "CSV",
    command: "flysoar search -o NYC -d LON -D 2026-08-01 -O csv -n 20 > flights.csv",
    highlights: ["-O csv", "> flights.csv"],
  },
  {
    label: "JSON input",
    command: "flysoar search --input '{\"origin\":\"SFO\",\"destination\":\"JFK\",\"date\":\"2026-07-15\"}'",
    highlights: ["--input"],
  },
  {
    label: "Save",
    command: "flysoar search -o SFO -d JFK -D 2026-07-15 --save results.json",
    highlights: ["--save results.json"],
  },
  {
    label: "Piped",
    command: "flysoar search -o NYC -d LON -D 2026-08-01 -q | jq '.offers[0].price'",
    highlights: ["-q"],
  },
];

const jsonInputExample = {
  origin: "SFO",
  destination: "JFK",
  date: "2026-07-15",
  cabin: "business",
  passengers: 1,
};

const flightRows = [
  { price: "$368", airline: "American Airlines", route: "SFO → JFK", stops: "0", duration: "5h 28m" },
  { price: "$389", airline: "JetBlue", route: "SFO → JFK", stops: "0", duration: "5h 34m" },
  { price: "$402", airline: "Delta", route: "SFO → JFK", stops: "0", duration: "5h 41m" },
  { price: "$451", airline: "United", route: "SFO → JFK", stops: "1", duration: "7h 12m" },
  { price: "$476", airline: "Alaska Airlines", route: "SFO → JFK", stops: "1", duration: "7h 44m" },
  { price: "$493", airline: "Sun Country", route: "SFO → JFK", stops: "1", duration: "8h 03m" },
];

function useReveal() {
  const [revealed, setRevealed] = useState(() => new Set());

  useEffect(() => {
    const items = document.querySelectorAll("[data-reveal]");
    const reveal = (item) => {
      const key = item.dataset.reveal;
      setRevealed((current) => {
        if (current.has(key)) return current;
        const next = new Set(current);
        next.add(key);
        return next;
      });
    };

    if (typeof IntersectionObserver === "undefined") {
      items.forEach(reveal);
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            reveal(entry.target);
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.14, rootMargin: "0px 0px -8% 0px" }
    );
    items.forEach((el) => observer.observe(el));
    return () => observer.disconnect();
  }, []);

  return revealed;
}

function TerminalDemo() {
  const [started, setStarted] = useState(false);
  const terminalRef = useRef(null);

  useEffect(() => {
    const terminal = terminalRef.current;
    if (!terminal || typeof IntersectionObserver === "undefined") {
      setStarted(true);
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setStarted(true);
          observer.disconnect();
        }
      },
      { threshold: 0.35 }
    );

    observer.observe(terminal);
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={terminalRef} className={`terminal${started ? " terminal-running" : ""}`} role="img" aria-label="Animated example of a flysoar flight search">
      <div className="terminal-bar">
        <span className="dot red" />
        <span className="dot yellow" />
        <span className="dot green" />
        <span className="terminal-title">flysoar — zsh</span>
      </div>
      <div className="terminal-body">
        <div className="terminal-command">
          <span className="terminal-dot" aria-hidden="true" />
          <span className="prompt">$</span>{" "}
          {started && (
            <TextType
              as="span"
              text="flysoar search -o SFO -d JFK -D 2026-07-15 -O table"
              typingSpeed={12}
              initialDelay={180}
              loop={false}
              showCursor={false}
            />
          )}
        </div>

        <div className="terminal-searching">
          searching flysoar.ai<span className="search-dots" aria-hidden="true"><i>.</i><i>.</i><i>.</i></span>
        </div>

        <div className="flight-table">
          <div className="flight-row flight-header">
            <span>PRICE</span><span>AIRLINE</span><span>ROUTE</span><span>STOPS</span><span>DURATION</span>
          </div>
          <div className="flight-viewport">
            <div className="flight-track">
              {[...flightRows, ...flightRows].map((flight, index) => (
                <div className="flight-row" key={`${flight.airline}-${flight.price}-${index}`} aria-hidden={index >= flightRows.length}>
                  <span className="price">{flight.price}</span>
                  <span>{flight.airline}</span>
                  <span>{flight.route}</span>
                  <span>{flight.stops}</span>
                  <span>{flight.duration}</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="terminal-live">
          <span className="ok">✓ done</span>{" "}<span className="dim">in 2.4s</span><span className="terminal-end-cursor" aria-hidden="true">_</span>
        </div>
      </div>
    </div>
  );
}

function JsonTree({ highlights }) {
  return (
    <code className="json-command" role="tree" aria-label="JSON input command">
      <span className="json-line"><span className="prompt">$</span> <TypedCommand command="flysoar search --input '" highlights={highlights} typingSpeed={5} initialDelay={0} showCursor={false} /></span>
      <span className="json-line"><span className="json-brace"><TextType as="span" text="{" typingSpeed={5} initialDelay={150} loop={false} showCursor={false} /></span></span>
        {Object.entries(jsonInputExample).map(([key, value], index) => (
          <span className="json-line json-child" key={key} role="treeitem">
            <TextType
              as="span"
              text={`"${key}": ${typeof value === "string" ? `"${value}"` : value}${index < Object.keys(jsonInputExample).length - 1 ? "," : ""}`}
              typingSpeed={5}
              initialDelay={260 + index * 150}
              loop={false}
              showCursor={index === Object.keys(jsonInputExample).length - 1}
              hideCursorWhileTyping
              cursorCharacter="|"
              cursorBlinkDuration={0.7}
              cursorClassName="cursor"
              className="json-value"
            />
          </span>
        ))}
      <span className="json-line"><span className="json-brace"><TextType as="span" text="}'" typingSpeed={5} initialDelay={190} loop={false} showCursor={false} /></span></span>
    </code>
  );
}

function TypedCommand({ command, highlights, typingSpeed = 5, initialDelay = 30, showCursor = true }) {
  const [revealCount, setRevealCount] = useState(0);

  useEffect(() => {
    setRevealCount(0);
    let interval;
    const startTimer = window.setTimeout(() => {
      let i = 0;
      interval = window.setInterval(() => {
        i += 1;
        setRevealCount(i);
        if (i >= command.length) {
          window.clearInterval(interval);
        }
      }, typingSpeed);
    }, initialDelay);

    return () => {
      window.clearTimeout(startTimer);
      window.clearInterval(interval);
    };
  }, [command, typingSpeed, initialDelay]);

  const revealedText = command.slice(0, revealCount);
  const done = revealCount >= command.length;

  // Resolve each highlight substring to a [start, end) range within `command`,
  // sorted and de-overlapped so multiple highlighted parts render correctly.
  const ranges = (highlights || [])
    .map((part) => {
      const start = command.indexOf(part);
      return start === -1 ? null : { start, end: start + part.length };
    })
    .filter(Boolean)
    .sort((a, b) => a.start - b.start);

  let content;
  if (ranges.length === 0) {
    content = revealedText;
  } else {
    const pieces = [];
    let cursor = 0;
    ranges.forEach((range, i) => {
      const start = Math.min(range.start, revealedText.length);
      const end = Math.min(range.end, revealedText.length);
      if (start > cursor) {
        pieces.push(revealedText.slice(cursor, start));
      }
      const highlighted = revealedText.slice(start, end);
      if (highlighted) {
        pieces.push(<span className="flag-highlight" key={i}>{highlighted}</span>);
      }
      cursor = Math.max(cursor, end);
    });
    if (cursor < revealedText.length) {
      pieces.push(revealedText.slice(cursor));
    }
    content = pieces;
  }

  return (
    <span className="text-type">
      <span className="text-type-content">{content}</span>
      {showCursor && done && (
        <span className="example-cursor" aria-hidden="true">|</span>
      )}
    </span>
  );
}

export default function Home() {
  const [copied, setCopied] = useState(false);
  const [activeExample, setActiveExample] = useState(0);
  const [exampleCopied, setExampleCopied] = useState(false);
  const [exampleKey, setExampleKey] = useState(0);
  const backgroundRef = useRef(null);
  const isJsonExample = commandExamples[activeExample].label === "JSON input";

  const revealed = useReveal();
  const revealClass = (key) => (revealed.has(key) ? " is-visible" : "");

  useEffect(() => {
    function onMove(e) {
      const background = backgroundRef.current;
      if (background) {
        const x = e.clientX / window.innerWidth - 0.5;
        const y = e.clientY / window.innerHeight - 0.5;
        background.style.setProperty("--bg-x", `${x * -10}px`);
        background.style.setProperty("--bg-y", `${y * -7}px`);
      }
    }

    window.addEventListener("pointermove", onMove);
    return () => {
      window.removeEventListener("pointermove", onMove);
    };
  }, []);

  useEffect(() => {
    // All glowing dots across the page (hero, terminal demo, example box)
    // share the same animation duration. Resetting them together on mount
    // forces every instance to restart its pulse on the exact same tick,
    // so they all stay perfectly in phase with each other.
    const dots = document.querySelectorAll(".terminal-dot");
    dots.forEach((dot) => {
      dot.style.animation = "none";
    });
    void document.body.offsetHeight;
    dots.forEach((dot) => {
      dot.style.animation = "";
    });
  }, []);

  async function copyInstallCommand() {
    await navigator.clipboard.writeText(installCommand);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1800);
  }

  async function copyExample() {
    await navigator.clipboard.writeText(commandExamples[activeExample].command);
    setExampleCopied(true);
    window.setTimeout(() => setExampleCopied(false), 1600);
  }

  return (
    <>
      <div className="bg" ref={backgroundRef} aria-hidden="true">
        <LiveFlightScene />
        <div className="image-vignette" />
      </div>

      <main>
        <nav className="nav" aria-label="Primary navigation">
          <div className="nav-brand">
            <span className="wordmark">Flysoar CLI</span>
            <span className="powered-by">
              Powered by <a href="https://flysoar.ai" target="_blank" rel="noreferrer">flysoar.ai</a>
            </span>
          </div>
        </nav>

        <section className="hero" id="top">
          <a
            href="https://github.com/Gahnxd/flysoar-cli"
            target="_blank"
            rel="noreferrer"
            data-reveal="eyebrow"
            className={`eyebrow${revealClass("eyebrow")}`}
          >
            <span className="ping" aria-hidden="true" />
            Open-source flight search CLI
            <span className="eyebrow-arrow" aria-hidden="true">→</span>
          </a>
          <h1 data-reveal="headline" className={revealClass("headline").trim()}>
            Flights, without <br className="br" />the friction.
          </h1>
          <p className={`lede${revealClass("lede")}`} data-reveal="lede">
            A clean command-line interface for searching live flight offers. Built for agents, scripts, and people who live in the terminal.
          </p>

          <div className={`install-card${revealClass("install")}`} data-reveal="install">
            <div className="terminal-dot" aria-hidden="true" />
            <code><span>$</span> <TextType as="span" text={installCommand} typingSpeed={5} initialDelay={160} loop={false} startOnVisible hideCursorWhileTyping cursorCharacter="|" cursorBlinkDuration={0.7} cursorClassName="cursor" /></code>
            <button
              type="button"
              onClick={copyInstallCommand}
              aria-label={copied ? "Copied" : "Copy install command"}
              aria-live="polite"
            >
              {copied ? "✓" : "Copy"}
            </button>
          </div>

          <div className={`actions${revealClass("actions")}`} data-reveal="actions">
            <a
              className="button primary"
              href="https://github.com/Gahnxd/flysoar-cli/releases/latest"
              target="_blank"
              rel="noreferrer"
            >
              Latest release
            </a>
            <a
              className="button secondary"
              href="https://github.com/Gahnxd/flysoar-cli"
              target="_blank"
              rel="noreferrer"
            >
              <svg className="github-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z" />
              </svg>
              GitHub
            </a>
          </div>

          <p className={`availability${revealClass("availability")}`} data-reveal="availability">Available for macOS, Linux, and Windows</p>

          <div className={`hero-terminal${revealClass("terminal")}`} data-reveal="terminal">
            <TerminalDemo />
          </div>
        </section>

        <section className="section" id="usage" aria-label="Usage examples">
          <div className={`usage-card${revealClass("usage-card")}`} data-reveal="usage-card">
            <div className="usage-tabs" role="tablist" aria-label="Command examples">
              {commandExamples.map((ex, i) => (
                <button
                  key={ex.label}
                  type="button"
                  role="tab"
                  aria-selected={activeExample === i}
                  className={`usage-tab ${activeExample === i ? "active" : ""}`}
                  onClick={() => {
                setActiveExample(i);
                setExampleKey(k => k + 1);
              }}
                >
                  {ex.label}
                </button>
              ))}
            </div>
            <div className={`usage-command${isJsonExample ? " json-command-panel" : ""}`}>
              <div className="terminal-dot" aria-hidden="true" />
              {isJsonExample ? (
                <JsonTree highlights={commandExamples[activeExample].highlights} />
              ) : (
                <code>
                  <span className="prompt">$</span>{" "}
                  <TypedCommand key={exampleKey} command={commandExamples[activeExample].command} highlights={commandExamples[activeExample].highlights} />
                </code>
              )}
              <button
                type="button"
                className="usage-copy"
                onClick={copyExample}
                aria-label={exampleCopied ? "Copied" : "Copy example command"}
                aria-live="polite"
              >
                {exampleCopied ? "✓" : "Copy"}
              </button>
            </div>
          </div>
        </section>

        <footer>
          <span className="footer-copy">
            Not affiliated with or endorsed by Soar AI · Powered by{" "}
            <a href="https://flysoar.ai" target="_blank" rel="noreferrer">flysoar.ai</a>
          </span>
          <div className="footer-links">
            <a href="https://github.com/Gahnxd/flysoar-cli" target="_blank" rel="noreferrer">GitHub</a>
            <a href="https://github.com/Gahnxd/flysoar-cli/releases" target="_blank" rel="noreferrer">Releases</a>
          </div>
        </footer>
      </main>
    </>
  );
}
