import { ImageResponse } from "next/og";

export const alt = "Kipferl 0.7 — From Python script to shipped tool";
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
        background: "#101916",
        color: "#ecf3e8",
        padding: "60px 70px",
        fontFamily: "sans-serif",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          fontSize: 23,
        }}
      >
        <span style={{ fontWeight: 700 }}>Kipferl</span>
        <span style={{ color: "#9fdac1", fontSize: 17 }}>0.7 · STABLE</span>
      </div>
      <div
        style={{
          display: "flex",
          marginTop: 75,
          gap: 45,
          alignItems: "center",
        }}
      >
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            width: 620,
            fontSize: 68,
            lineHeight: 1.07,
            letterSpacing: "-3px",
          }}
        >
          <span>From Python script</span>
          <span style={{ color: "#9fdac1" }}>to shipped tool.</span>
        </div>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            width: 315,
            padding: "30px 25px",
            border: "1px solid #3b5143",
            borderRadius: 13,
            background: "#1b2922",
            gap: 19,
            fontSize: 20,
            color: "#c5e7b3",
            fontFamily: "monospace",
          }}
        >
          <span>$ kipferl new hello</span>
          <span>$ kipferl test</span>
          <span>$ kipferl build</span>
          <span style={{ color: "#ecf3e8" }}>Ready to ship.</span>
        </div>
      </div>
      <div
        style={{
          display: "flex",
          marginTop: "auto",
          paddingTop: 27,
          borderTop: "1px solid #304038",
          justifyContent: "space-between",
          color: "#a6b6ac",
          fontSize: 18,
        }}
      >
        <span>Python-style code. Checked packages. One executable.</span>
        <span>kipferl.dev</span>
      </div>
    </div>,
    size,
  );
}
