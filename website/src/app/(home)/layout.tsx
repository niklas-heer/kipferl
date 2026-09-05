import { HomeLayout } from "fumadocs-ui/layouts/home";
import type { Metadata } from "next";
import { baseOptions } from "@/lib/layout.shared";

export const metadata: Metadata = {
  title: { absolute: "Kipferl — From Python script to shipped tool" },
  description:
    "Create a project, check and lock PyPI packages, and ship a standalone CLI with Python-style code. See the complete Kipferl 0.7 workflow in action.",
};

export default function Layout({ children }: LayoutProps<"/">) {
  return <HomeLayout {...baseOptions()}>{children}</HomeLayout>;
}
