import Script from 'next/script'
import React from 'react'

function TrackingScript() {
  return (
    <>
      {process.env.NODE_ENV === 'production' && (
        <Script
          id="track"
          dangerouslySetInnerHTML={{
            __html: `
              (function () {
                var el = document.createElement('script');
                el.setAttribute('src', 'https://trail.learningman.top/script.js');
                el.setAttribute('data-website-id', '78c14a46-3fc8-4a5d-a643-e7079783384b');
                document.body.appendChild(el);
              })();
            `,
          }}
        />
      )}
    </>
  )
}

export default TrackingScript