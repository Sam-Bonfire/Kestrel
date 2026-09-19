// Single edit point for links and release targets.
// Mail and Calendar ship as separate installers in the same GitHub release;
// point each at its own asset URL once filenames are final (defaults: latest release page).
export const site = {
  name: 'Kestrel',
  tagline: 'Private mail + calendar suite',
  email: 'hello@example.com',
  github: 'https://github.com/Sam-Bonfire/Kestrel',
  mailDownload: 'https://github.com/Sam-Bonfire/Kestrel/releases/latest',
  calendarDownload: 'https://github.com/Sam-Bonfire/Kestrel/releases/latest',
  releaseNotes: 'https://github.com/Sam-Bonfire/Kestrel/releases/latest',
  dockerImage: 'ghcr.io/sam-bonfire/kestrel:latest',
  buildNotes: '',
} as const;

export interface PlatformBuild {
  os: string;
  format: string;
}

export const platforms: PlatformBuild[] = [
  { os: 'Windows', format: 'MSI / EXE' },
  { os: 'macOS', format: 'DMG (arm64)' },
  { os: 'Linux', format: '.deb / AppImage' },
  { os: 'Android', format: 'APK' },
  { os: 'iOS', format: 'TestFlight' },
];
