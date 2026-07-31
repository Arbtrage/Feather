import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Feather Dashboard",
  description: "Feather activity queue — read-only dashboard",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <header style={{ borderBottom: "1px solid #2a3544", padding: "1rem 2rem" }}>
          <nav style={{ display: "flex", gap: "1.5rem", alignItems: "center" }}>
            <strong>Feather</strong>
            <a href="/">Overview</a>
            <a href="/jobs">Jobs</a>
          </nav>
        </header>
        <main style={{ maxWidth: 1100, margin: "0 auto", padding: "2rem" }}>{children}</main>
      </body>
    </html>
  );
}
