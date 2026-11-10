import Script from 'next/script'
export default function TrackingScript() {
  if (process.env.NODE_ENV !== 'production') return null
  return (
    <>
      <Script
        id="umami"
        strategy="afterInteractive"
        src="https://trail.learningman.top/script.js"
        data-website-id="78c14a46-3fc8-4a5d-a643-e7079783384b"
        data-domains="report.cj.zxilly.dev"
        data-exclude-search="true"
        data-exclude-hash="true"
        data-before-send="reportBeforeSend"
      />
    </>
  )
}
