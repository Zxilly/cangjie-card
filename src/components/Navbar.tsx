import Link from 'next/link'

export function Navbar() {
  return (
    <nav className="bg-white border-b border-gray-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex">
            <div className="flex-shrink-0 flex items-center">
              <Link href="/" className="text-xl font-semibold">
                仓禀
              </Link>
            </div>
          </div>
          <div className="flex items-center space-x-6">
            <Link href="https://github.com/Zxilly/cangjie-report" className="text-sm text-gray-600 hover:text-gray-900" target="_blank" rel="noopener noreferrer">
              GitHub
            </Link>
          </div>
        </div>
      </div>
    </nav>
  )
} 