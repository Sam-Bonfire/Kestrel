// Single edit point for personal details, links, and release targets.
export const site = {
  name: 'Sam',
  role: 'Product Manager',
  email: 'hello@example.com',
  github: 'https://github.com/Sam-Bonfire/Kestrel',
  linkedin: 'https://www.linkedin.com/',
  repoReleases: 'https://github.com/Sam-Bonfire/Kestrel/releases/latest',
  dockerImage: 'ghcr.io/sam-bonfire/kestrel:latest',
} as const;

export interface PlatformBuild {
  os: string;
  format: string;
  note: string;
}

export const platforms: PlatformBuild[] = [
  { os: 'Windows', format: 'MSI / EXE', note: 'Mail + Calendar installers' },
  { os: 'macOS', format: 'DMG (arm64)', note: 'Mail + Calendar apps' },
  { os: 'Linux', format: '.deb / AppImage', note: 'Mail + Calendar packages' },
  { os: 'Android', format: 'APK', note: 'Mail + Calendar apps' },
  { os: 'iOS', format: 'TestFlight', note: 'Mail + Calendar builds' },
];
