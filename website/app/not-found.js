"use client";

import { useEffect, useRef, useState } from "react";
import { usePathname } from "next/navigation";
import Link from "next/link";
import TextType from "../components/TextType";
import LiveFlightScene from "../components/LiveFlightScene";

export default function NotFound() {
  const backgroundRef = useRef(null);
  const pathname = usePathname();
  const [commandText, setCommandText] = useState("flysoar search /");

  useEffect(() => {
    let decoded;
    try {
      decoded = decodeURIComponent(pathname);
    } catch {
      decoded = pathname;
    }
    const sanitized = decoded.replace(/[^a-zA-Z0-9\-_./~ ]/g, "").slice(0, 200);
    setCommandText(`flysoar search ${sanitized || "/"}`);
  }, [pathname]);

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

  return (
    <>
      <div className="bg" ref={backgroundRef} aria-hidden="true">
        <LiveFlightScene />
        <div className="image-vignette" />
      </div>

      <main className="not-found">
        <div className="not-found-terminal">
          <div className="not-found-bar">
            <span className="dot red" />
            <span className="dot yellow" />
            <span className="dot green" />
            <span className="not-found-title">flysoar — zsh</span>
          </div>
          <div className="not-found-body">
            <div className="not-found-command">
              <span className="prompt">$</span>{" "}
              <TextType
                as="span"
                text={commandText}
                typingSpeed={12}
                initialDelay={180}
                loop={false}
                showCursor={false}
              />
            </div>
            <div className="not-found-error">
              <TextType
                as="span"
                text="404 No flights found at this URL."
                typingSpeed={30}
                initialDelay={900}
                loop={false}
                showCursor={false}
              />
            </div>
            <div className="not-found-hint">
              The page you&apos;re looking for doesn&apos;t exist or has been moved.
            </div>
            <Link href="/" className="not-found-link">
              ← Back to Flysoar CLI
            </Link>
          </div>
        </div>
      </main>
    </>
  );
}
