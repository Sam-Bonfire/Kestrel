// Single edit point for links and release targets. Every identity value is
// resolved at build time (explicit KESTREL_* env, else git remote, else a
// generic placeholder) so forks ship this page with their own links.
const repo: string = __KESTREL_REPO__;
const github = `https://github.com/${repo}`;
const latestRelease = `${github}/releases/latest`;

export const site = {
  name: 'Kestrel',
  tagline: 'Private mail + calendar suite',
  repo,
  github,
  mailDownload: latestRelease,
  calendarDownload: latestRelease,
  releaseNotes: latestRelease,
  dockerImage:
    __KESTREL_DOCKER_IMAGE__ !== '' ? __KESTREL_DOCKER_IMAGE__ : `ghcr.io/${repo.toLowerCase()}:latest`,
  buildNotes: '',
  // Contact email shown in the footer; empty hides the link entirely.
  contactEmail: __KESTREL_CONTACT_EMAIL__ as string,
  // TestFlight invite URL once the iOS beta opens; empty renders "invite only".
  iosTestFlight: __KESTREL_IOS_TESTFLIGHT__ as string,
};

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
