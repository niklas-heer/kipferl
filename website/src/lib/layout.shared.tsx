import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import Image from "next/image";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="flex items-center gap-2 font-bold">
          <Image
            src="/kipferl-logo.png"
            alt=""
            width={28}
            height={28}
            className="size-7"
          />
          <span>Kipferl</span>
        </span>
      ),
    },
    links: [
      {
        text: "Docs",
        url: "/docs",
      },
      {
        text: "Contribute",
        url: "/docs/guides/development",
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
