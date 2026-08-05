import { Analytics } from "@vercel/analytics/next";
import { RootProvider } from "fumadocs-ui/provider/next";
import "./global.css";
import type { Metadata } from "next";
import { Inter } from "next/font/google";

export const metadata: Metadata = {
  metadataBase: new URL("https://getkipferl.org"),
  title: {
    default: "Kipferl — Python CLIs, standalone binaries",
    template: "%s | Kipferl",
  },
  description:
    "Build beautiful command-line applications with Python syntax and ship them as fast, standalone binaries.",
};

const inter = Inter({
  subsets: ["latin"],
});

export default function Layout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en" className={inter.className} suppressHydrationWarning>
      <body className="flex flex-col min-h-screen">
        <RootProvider>{children}</RootProvider>
        <Analytics />
      </body>
    </html>
  );
}
