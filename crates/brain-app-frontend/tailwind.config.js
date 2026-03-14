/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "class",
  content: ["./src/**/*.rs", "./index.html"],
  theme: {
    extend: {
      colors: {
        surface: "var(--color-surface)",
        accent: "var(--color-accent)",
        "type-learning": "var(--color-type-learning)",
        "type-gotcha": "var(--color-type-gotcha)",
        "type-project": "var(--color-type-project)",
      },
    },
  },
  plugins: [require("@tailwindcss/typography")],
};
