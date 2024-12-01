/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["./crates/*/src/**/*.rs"],
  },
  theme: {
    extend: {
      gridTemplateColumns: {
        'main': '70px 1fr',
      },
    },
  },
  plugins: [],
}