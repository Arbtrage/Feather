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
        text: "PyPI",
        url: "https://pypi.org/project/getfeather/",
        external: true,
      },
    ],
  };
}
