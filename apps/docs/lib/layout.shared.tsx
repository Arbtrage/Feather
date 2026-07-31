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
        text: "Packages",
        url: "https://github.com/Arbtrage/Feather/pkgs/npm/feather",
        external: true,
      },
      {
        text: "Releases",
        url: "https://github.com/Arbtrage/Feather/releases",
        external: true,
      },
    ],
  };
}
