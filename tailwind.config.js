/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["./crates/*/src/**/*.rs"],
  },
  theme: {
    extend: {
      gridTemplateColumns: {
        'main': '50px 1fr',
      },
    },
  },
  plugins: [],
}