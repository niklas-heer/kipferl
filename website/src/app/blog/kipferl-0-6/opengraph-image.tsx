import { ImageResponse } from "next/og";

export const alt = "Kipferl 0.6 — Rust-powered Python CLIs, baked smaller";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OpenGraphImage() {
  return new ImageResponse(
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        padding: "70px 78px",
        color: "white",
        backgroundColor: "#030712",
        backgroundImage:
          "radial-gradient(circle at 86% 10%, #7c3aed 0, transparent 34%), radial-gradient(circle at 10% 90%, #0891b2 0, transparent 38%)",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 18,
          fontSize: 28,
          color: "#67e8f9",
          letterSpacing: 2,
        }}
      >
        <span style={{ fontSize: 52 }}>☾</span>
        KIPFERL 0.6 · STABLE RELEASE
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
        <div
          style={{
            display: "flex",
            fontSize: 68,
            fontWeight: 800,
            lineHeight: 1.05,
            maxWidth: 1000,
          }}
        >
          Rust-powered Python CLIs, baked smaller
        </div>
        <div style={{ display: "flex", fontSize: 30, color: "#cbd5e1" }}>
          Standalone apps from 1.4 MB · 7.679 ms median startup
        </div>
      </div>
      <div style={{ display: "flex", fontSize: 25, color: "#94a3b8" }}>
        kipferl.dev
      </div>
    </div>,
    size,
  );
}
