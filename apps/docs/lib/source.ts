import { docs as docsCollection } from "@/.source";
import { loader } from "fumadocs-core/source";
import { resolveFiles } from "fumadocs-mdx";

export const source = loader({
  baseUrl: "/docs",
  source: {
    files: resolveFiles({
      docs: docsCollection.docs,
      meta: docsCollection.meta,
    }),
  },
});
