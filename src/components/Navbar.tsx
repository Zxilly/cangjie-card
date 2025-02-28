import Link from 'next/link'

export function Navbar() {
  return (
    <nav className="fixed top-0 left-0 right-0 bg-white border-b border-gray-200 z-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex">
            <div className="flex-shrink-0 flex items-center">
              <Link href="/" className="text-xl font-semibold">
                Cangjie Card
              </Link>
            </div>
          </div>
          <div className="flex items-center space-x-6">
            <Link href="https://github.com/Zxilly/cangjie-card" className="text-sm text-gray-600 hover:text-gray-900">
              GitHub
            </Link>
            <Link href="/about" className="text-sm text-gray-600 hover:text-gray-900">
              About
            </Link>
          </div>
        </div>
      </div>
    </nav>
  )
} 