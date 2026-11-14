import { createMDX } from "fumadocs-mdx/next";

const withMDX = createMDX();

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  async redirects() {
    return ["0.7", "0.7.1"].flatMap((version) =>
      ["mp4", "webp"].map((extension) => ({
        source: `/demos/kipferl-${version}.${extension}`,
        destination: `/demos/kipferl-0.9.0.${extension}`,
        permanent: true,
      })),
    );
  },
};

export default withMDX(config);
