import type { Metadata } from "next";
import Script from "next/script";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Navbar } from "@/components/Navbar";
import TrackingScript from "@/components/TrackingScript";
const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  metadataBase: new URL("https://report.cj.zxilly.dev"),
  title: {
    default: "仓禀：仓颉代码质量分析与 cjlint 报告",
    template: "%s | 仓禀",
  },
  description: "输入仓颉项目的 Git 仓库地址，使用仓禀查看 cjlint 检查结果、问题分级与代码质量评分，并生成可嵌入项目文档的质量徽章。",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <Navbar />
        {children}
        {process.env.NODE_ENV === "production" && (
          <Script id="analytics-filter" strategy="beforeInteractive">{`
        window.reportBeforeSend = function (_type, payload) {
          var page = new URL(payload.url || '/', window.location.origin);
          if (page.pathname !== '/') return false;
          payload.url = '/';
          payload.title = '仓禀：仓颉代码质量分析与 cjlint 报告';
          if (payload.referrer) {
            try {
              var referrer = new URL(payload.referrer);
              payload.referrer = referrer.origin;
            } catch (_) {
              payload.referrer = '';
            }
          }
          return payload;
        };
          `}</Script>
        )}
        <TrackingScript />
      </body>
    </html>
  );
}
