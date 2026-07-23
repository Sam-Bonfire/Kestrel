export const colors = {
  canvasBase: '#0D0D0D',
  canvasCard: '#131313',
  canvasHover: '#1C1B1B',
  canvasModal: '#2A2A2A',
  textPrimary: '#E5E2E1',
  textSecondary: '#A0A0A0',
  borderHairline: '#353534',
  primary: '#FFFFFF',
  primaryMuted: 'rgba(255, 255, 255, 0.15)',
  accentGmail: '#EA4335',
  accentOutlook: '#0078D4',
  todayRed: '#D15B47',
  starredAmber: '#E5B722',
} as const;

export const tagColors = {
  devops: {
    text: '#60a5fa',
    bg: 'rgba(30, 58, 138, 0.3)',
    border: 'rgba(30, 58, 138, 0.3)',
  },
  careers: {
    text: '#34d399',
    bg: 'rgba(6, 78, 59, 0.3)',
    border: 'rgba(6, 78, 59, 0.3)',
  },
  finance: {
    text: '#c084fc',
    bg: 'rgba(88, 28, 135, 0.3)',
    border: 'rgba(88, 28, 135, 0.3)',
  },
  urgent: {
    text: '#d15b47',
    bg: 'rgba(127, 29, 29, 0.2)',
    border: 'rgba(153, 27, 27, 0.3)',
  },
} as const;

export const typography = {
  displayTitle1: {
    fontFamily: 'sans',
    fontWeight: '700',
    letterSpacing: '-0.025em',
    fontSize: '1.5rem',
    lineHeight: '2rem',
  },
  sectionHeader: {
    fontFamily: 'sans',
    fontWeight: '500',
    fontSize: '0.75rem',
    color: 'textSecondary',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.1em',
  },
  emailTitleRow: {
    fontFamily: 'sans',
    fontWeight: '600',
    fontSize: '0.875rem',
    lineHeight: '1.25rem',
  },
  body: {
    fontFamily: 'sans',
    fontWeight: '400',
    fontSize: '0.75rem',
    lineHeight: '1.625',
    color: 'textSecondary',
  },
  monospace: {
    fontFamily: 'mono',
    fontWeight: '400',
    fontSize: '0.75rem',
    lineHeight: '1rem',
  },
  miniBadge: {
    fontFamily: 'mono',
    fontWeight: '400',
    fontSize: '10px',
    lineHeight: '14px',
    color: 'textSecondary',
    opacity: 0.6,
  },
} as const;

export const fonts = {
  sans: '"Geist", ui-sans-serif, system-ui, sans-serif',
  mono: '"JetBrains Mono", ui-monospace, SFMono-Regular, monospace',
} as const;

export const spacing = {
  xs: '0.25rem',
  sm: '0.5rem',
  md: '1rem',
  lg: '1.5rem',
  xl: '2rem',
} as const;

export const radius = {
  sm: '4px',
  md: '6px',
  lg: '8px',
  full: '9999px',
} as const;

export const shadows = {
  sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
  md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
  lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
} as const;

export const borders = {
  hairline: { width: '1px', color: colors.borderHairline },
  focus: { width: '2px', color: colors.primary },
} as const;

export const buttonStyles = {
  primary: {
    bg: colors.primary,
    hoverBg: '#e5e5e5',
    textColor: '#000000',
    fontSize: '0.75rem',
    fontWeight: '600',
    paddingX: '0.75rem',
    paddingY: '0.375rem',
    borderRadius: radius.lg,
  },
  secondary: {
    bg: colors.canvasCard,
    hoverBg: colors.canvasHover,
    borderColor: colors.borderHairline,
    textColor: colors.textPrimary,
    fontSize: '0.75rem',
    borderRadius: radius.lg,
  },
  ghost: {
    padding: '0.375rem',
    borderRadius: radius.lg,
    hoverBg: 'rgba(255, 255, 255, 0.05)',
    textColor: colors.textSecondary,
    hoverTextColor: colors.textPrimary,
  },
  danger: {
    padding: '0.375rem',
    borderRadius: radius.lg,
    hoverBg: 'rgba(127, 29, 29, 0.4)',
    textColor: '#f87171',
    hoverTextColor: '#fca5a5',
    borderColor: 'rgba(153, 27, 27, 0.2)',
  },
} as const;

export const animations = {
  fadeIn: {
    keyframes: { from: { opacity: '0' }, to: { opacity: '1' } },
    duration: '200ms',
    easing: 'ease-out',
  },
  scaleIn: {
    keyframes: {
      from: { opacity: '0', transform: 'scale(0.95)' },
      to: { opacity: '1', transform: 'scale(1)' },
    },
    duration: '150ms',
    easing: 'ease-out',
  },
  spinSlow: {
    keyframes: {
      from: { transform: 'rotate(0deg)' },
      to: { transform: 'rotate(360deg)' },
    },
    duration: '2s',
    easing: 'linear',
  },
  slideDown: {
    keyframes: {
      from: { opacity: '0', transform: 'translateY(-8px)' },
      to: { opacity: '1', transform: 'translateY(0)' },
    },
    duration: '200ms',
    easing: 'ease-out',
  },
  slideLeft: {
    keyframes: {
      from: { opacity: '0', transform: 'translateX(8px)' },
      to: { opacity: '1', transform: 'translateX(0)' },
    },
    duration: '200ms',
    easing: 'ease-out',
  },
  slideUp: {
    keyframes: {
      from: { opacity: '0', transform: 'translateY(8px)' },
      to: { opacity: '1', transform: 'translateY(0)' },
    },
    duration: '200ms',
    easing: 'ease-out',
  },
} as const;

/** CSS keyframes string for embedding in <style> tags */
export const keyframesCSS = {
  fadeIn:
    '@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }',
  scaleIn:
    '@keyframes scaleIn { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }',
  spinSlow:
    '@keyframes spinSlow { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }',
  slideDown:
    '@keyframes slideDown { from { opacity: 0; transform: translateY(-8px); } to { opacity: 1; transform: translateY(0); } }',
  slideLeft:
    '@keyframes slideLeft { from { opacity: 0; transform: translateX(8px); } to { opacity: 1; transform: translateX(0); } }',
  slideUp:
    '@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }',
} as const;

/** Flat tokens object preserved for backwards compat */
export const tokens = {
  colors,
  tagColors,
  typography,
  fonts,
  spacing,
  radius,
  shadows,
  borders,
  buttonStyles,
  animations,
} as const;

export type DesignTokens = typeof tokens;
