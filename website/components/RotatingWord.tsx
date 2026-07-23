"use client";

import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { useEffect, useState } from "react";

const WORDS = ["codes", "device"];
const INTERVAL_MS = 2400;

export function RotatingWord() {
  const [index, setIndex] = useState(0);
  const reduceMotion = useReducedMotion();

  useEffect(() => {
    const id = setInterval(
      () => setIndex((i) => (i + 1) % WORDS.length),
      INTERVAL_MS,
    );
    return () => clearInterval(id);
  }, []);

  return (
    <span className="relative mx-1 inline-grid overflow-hidden border border-line bg-white/[0.04] px-3 md:px-5">
      {/* Widest word reserves the box size so the layout never shifts. */}
      <span aria-hidden className="invisible col-start-1 row-start-1">
        device
      </span>
      <AnimatePresence initial={false} mode="popLayout">
        <motion.span
          key={WORDS[index]}
          className="col-start-1 row-start-1 text-center"
          initial={reduceMotion ? { opacity: 0 } : { y: "-80%", opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          exit={reduceMotion ? { opacity: 0 } : { y: "80%", opacity: 0 }}
          transition={{ duration: 0.25, ease: "easeOut" }}
        >
          {WORDS[index]}
        </motion.span>
      </AnimatePresence>
    </span>
  );
}
