import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: "Feather",
    },
    links: [
      {
        text: "GitHub",
        url: "https://github.com/Arbtrage/Feather",
        external: true,
      },
      {
        text: "npm",
        url: "https://www.npmjs.com/package/@arbitrage/feather",
        external: true,
      },
      {
        text: "PyPI",
        url: "https://pypi.org/project/arbitrage-feather/",
        external: true,
      },
    ],
  };
}
