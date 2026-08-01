import { useState } from 'react'

function App() {
  const [count, setCount] = useState(0)

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-200">
      <h1 className="text-4xl font-bold mb-4">React + Vite + Tailwind</h1>
      <p className="mb-6 text-gray-500 dark:text-gray-400">
        Edit <code className="bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded text-sm">src/App.jsx</code> and save to test HMR
      </p>
      <button
        type="button"
        className="px-4 py-2 rounded bg-purple-600 text-white hover:bg-purple-700 transition-colors"
        onClick={() => setCount((c) => c + 1)}
      >
        Count is {count}
      </button>
    </div>
  )
}

export default App
