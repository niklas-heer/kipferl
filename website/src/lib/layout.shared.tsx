import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="font-bold">
          <span className="text-cyan-500">u</span>charm
        </span>
      ),
    },
    links: [
      {
        text: "Docs",
        url: "/docs",
      },
      {
        text: "Blog",
        url: "/blog",
      },
      {
        text: "GitHub",
        url: "https://github.com/niklas-heer/kipferl",
        external: true,
      },
    ],
    githubUrl: "https://github.com/niklas-heer/kipferl",
  };
}
