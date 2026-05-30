/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // MandingoForge luxurious dark theme - deep blacks, golds, blood reds
        'mf-black': {
          DEFAULT: '#0A0A0A',
          50: '#1A1A1A',
          100: '#121212',
          200: '#0F0F0F',
          300: '#0D0D0D',
          400: '#0A0A0A',
        },
        'mf-gold': {
          DEFAULT: '#C5A46E',
          50: '#F5F0E6',
          100: '#EDE4D3',
          200: '#D9C9A3',
          300: '#C5A46E',
          400: '#A67C4A',
          500: '#8B5E2F',
        },
        'mf-red': {
          DEFAULT: '#8B0000',
          50: '#3D0000',
          100: '#5C0000',
          200: '#8B0000',
          300: '#A52A2A',
          400: '#B22222',
        },
        'mf-cream': '#F5F0E6',
      },
      fontFamily: {
        'serif-display': ['Playfair Display', 'Georgia', 'serif'],
        'sans-body': ['Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
}