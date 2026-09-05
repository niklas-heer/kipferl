import { createMDX } from "fumadocs-mdx/next";

const withMDX = createMDX();

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  async redirects() {
    return ["mp4", "webp"].map((extension) => ({
      source: `/demos/kipferl-0.7.${extension}`,
      destination: `/demos/kipferl-0.7.1.${extension}`,
      permanent: true,
    }));
  },
};

export default withMDX(config);
