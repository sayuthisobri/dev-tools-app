import colors from 'tailwindcss/colors'

import {createThemes} from 'tw-colors'

import {fontFamily} from 'tailwindcss/defaultTheme'


const baseColors = [
  'gray',
  'red',
  'yellow',
  'orange',
  'green',
  'blue',
  'indigo',
  'purple',
  'pink',
]

const shadeMapping = {
  '50': '900',
  '100': '800',
  '200': '700',
  '300': '600',
  '400': '500',
  '500': '400',
  '600': '300',
  '700': '200',
  '800': '100',
  '900': '50',
}

const generateThemeObject = (colors: any, mapping: any, invert = false) => {
  const theme: any = {}
  baseColors.forEach((color) => {
    theme[color] = {}
    Object.entries(mapping).forEach(([key, value]) => {
      const shadeKey: any = invert ? value : key
      theme[color][key] = colors[color][shadeKey]
    })
  })
  return theme
}

const lightTheme = generateThemeObject(colors, shadeMapping)
const darkTheme = generateThemeObject(colors, shadeMapping, true)

const themes = {
  light: {
    ...lightTheme,
    white: '#ffffff',
  },
  dark: {
    ...darkTheme,
    white: colors.gray['950'],
    black: colors.gray['50'],
  },
}

module.exports = {
  darkMode: 'class',
  content: [
    // './renderer/app/**/*.{js,ts,jsx,tsx}',
    // './renderer/(providers)/**/*.{js,ts,jsx,tsx}',
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./lib/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    container: {
      center: true,
      padding: '2rem',
      screens: {
        '2xl': '1400px',
      },
    },
    colors: {
      // use colors only specified
      // white: colors.white,
      // gray: colors.gray,
      // blue: colors.blue,
    },
    extend: {
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic':
          'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
      },
      colors: {
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--twc-gray-50))',
        foreground: 'hsl(var(--foreground))',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        secondary: {
          DEFAULT: 'hsl(var(--secondary))',
          foreground: 'hsl(var(--secondary-foreground))',
        },
        destructive: {
          DEFAULT: 'hsl(var(--twc-red-200))',
          foreground: 'hsl(var(--twc-red-900))',
        },
        muted: {
          DEFAULT: 'hsl(var(--muted))',
          foreground: 'hsl(var(--muted-foreground))',
        },
        accent: {
          DEFAULT: 'hsl(var(--accent))',
          foreground: 'hsl(var(--accent-foreground))',
        },
        popover: {
          DEFAULT: 'hsl(var(--popover))',
          foreground: 'hsl(var(--popover-foreground))',
        },
        card: {
          DEFAULT: 'hsl(var(--card))',
          foreground: 'hsl(var(--card-foreground))',
        },
      },
      borderRadius: {
        lg: `var(--radius)`,
        md: `calc(var(--radius) - 2px)`,
        sm: 'calc(var(--radius) - 4px)',
      },
      fontFamily: {
        sans: ['var(--font-sans)', ...fontFamily.sans],
      },
      keyframes: {
        'accordion-down': {
          from: {height: '0'},
          to: {height: 'var(--radix-accordion-content-height)'},
        },
        'accordion-up': {
          from: {height: 'var(--radix-accordion-content-height)'},
          to: {height: '0'},
        },
      },
      animation: {
        'accordion-down': 'accordion-down 0.2s ease-out',
        'accordion-up': 'accordion-up 0.2s ease-out',
      },

    },
  },
  plugins: [
    createThemes(themes),
    require('tailwindcss-animate'),
    require('@tailwindcss/typography'),
    require('tailwind-scrollbar')({nocompatible: true}),
  ],
}
