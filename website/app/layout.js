import { Inter, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-sans",
  display: "swap",
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
  display: "swap",
});

export const metadata = {
  title: "flysoar cli — flight search from your terminal",
  description:
    "A fast, open-source CLI for searching live flight offers from your terminal. No accounts, no API keys. Powered by flysoar.ai.",
  metadataBase: new URL("https://flysoar.ai"),
  openGraph: {
    title: "flysoar cli — flight search from your terminal",
    description:
      "A fast, open-source CLI for searching live flight offers from your terminal. Powered by flysoar.ai.",
    type: "website",
  },
};

export const viewport = {
  themeColor: "#08080a",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en" data-scroll-behavior="smooth" className={`${inter.variable} ${jetbrainsMono.variable}`}>
      <body>{children}</body>
    </html>
  );
}
