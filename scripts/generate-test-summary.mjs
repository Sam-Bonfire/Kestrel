import fs from 'node:fs';
import path from 'node:path';

function readTextFile(filePath) {
  if (!fs.existsSync(filePath)) return null;
  const buf = fs.readFileSync(filePath);
  // Check for UTF-16LE BOM or null bytes
  if (buf.length >= 2 && buf[0] === 0xff && buf[1] === 0xfe) {
    return buf.toString('utf16le');
  }
  // Heuristic for UTF-16LE without BOM (common in PowerShell > redirection)
  if (buf.length >= 4 && buf[1] === 0x00 && buf[3] === 0x00) {
    return buf.toString('utf16le');
  }
  return buf.toString('utf8');
}

function parseCargoTestLog(logPath) {
  const content = readTextFile(logPath);
  if (!content) return null;

  const suites = [];
  let totalPassed = 0;
  let totalFailed = 0;
  let totalIgnored = 0;

  // Split into running blocks
  const blocks = content.split(/Running\s+/g);
  for (const block of blocks) {
    const lines = block.split(/\r?\n/);
    const firstLine = lines[0] ? lines[0].trim() : '';
    const resultMatch = block.match(/test result:\s+(\w+)\.\s+(\d+)\s+passed;\s+(\d+)\s+failed;\s+(\d+)\s+ignored;[^\r\n]*?finished in\s+([\d\.]+)s/);

    if (resultMatch) {
      const status = resultMatch[1];
      const passed = parseInt(resultMatch[2], 10);
      const failed = parseInt(resultMatch[3], 10);
      const ignored = parseInt(resultMatch[4], 10);
      const duration = resultMatch[5];

      // Extract a clean suite name from firstLine (e.g. unittests src/lib.rs or tests/api_handler_tests.rs)
      let name = firstLine.replace(/\s*\(.*?\)\s*/g, '').replace(/^unittests\s+/, '').trim();
      if (!name) name = 'cargo test suite';

      if (passed + failed + ignored > 0) {
        suites.push({
          name,
          status: status === 'ok' ? 'passed' : 'failed',
          passed,
          failed,
          ignored,
          duration: `${duration}s`
        });
        totalPassed += passed;
        totalFailed += failed;
        totalIgnored += ignored;
      }
    }
  }

  return {
    suites,
    totalPassed,
    totalFailed,
    totalIgnored,
    totalTests: totalPassed + totalFailed + totalIgnored
  };
}

function parseVitestJson(jsonPath) {
  const content = readTextFile(jsonPath);
  if (!content) return null;

  try {
    const data = JSON.parse(content);
    const suites = (data.testResults || []).map(r => {
      const relativePath = path.basename(r.name);
      const passed = (r.assertionResults || []).filter(a => a.status === 'passed').length;
      const failed = (r.assertionResults || []).filter(a => a.status === 'failed').length;
      const skipped = (r.assertionResults || []).filter(a => a.status === 'skipped' || a.status === 'pending').length;
      const durationMs = (r.assertionResults || []).reduce((acc, a) => acc + (a.duration || 0), 0);

      return {
        name: relativePath,
        status: r.status,
        passed,
        failed,
        skipped,
        duration: `${durationMs.toFixed(1)}ms`
      };
    });

    return {
      suites,
      totalPassed: data.numPassedTests || 0,
      totalFailed: data.numFailedTests || 0,
      totalPending: data.numPendingTests || 0,
      totalTests: data.numTotalTests || 0
    };
  } catch (e) {
    return null;
  }
}

export function generateMarkdownSummary() {
  const cargoResults = parseCargoTestLog('backend/test-results.log');
  const vitestResults = parseVitestJson('frontend-shared/test-results.json');

  let md = '';

  const totalRust = cargoResults ? cargoResults.totalTests : 0;
  const totalVitest = vitestResults ? vitestResults.totalTests : 0;
  const totalTests = totalRust + totalVitest;
  const totalFailed = (cargoResults ? cargoResults.totalFailed : 0) + (vitestResults ? vitestResults.totalFailed : 0);

  md += `## 🧪 Test Suite & Validation Summary\n\n`;

  md += `> **Status**: ${totalFailed === 0 ? '🟢 All Suites Passing' : '🔴 Test Failures Detected'} &nbsp;|&nbsp; ` +
        `**Total Tests**: \`${totalTests}\` &nbsp;|&nbsp; ` +
        `**Passed**: \`${totalTests - totalFailed}\` &nbsp;|&nbsp; ` +
        `**Failed**: \`${totalFailed}\`\n\n`;

  if (cargoResults && cargoResults.suites.length > 0) {
    md += `### 🦀 Rust Backend Test Suites (\`${cargoResults.totalPassed}\` passed, \`${cargoResults.totalFailed}\` failed)\n\n`;
    md += `| Test Suite | Status | Passed | Failed | Duration |\n`;
    md += `| :--- | :---: | :---: | :---: | :---: |\n`;
    for (const suite of cargoResults.suites) {
      const badge = suite.status === 'passed' ? '✅ Passed' : '❌ Failed';
      md += `| \`${suite.name}\` | ${badge} | ${suite.passed} | ${suite.failed} | ${suite.duration} |\n`;
    }
    md += `\n`;
  }

  if (vitestResults && vitestResults.suites.length > 0) {
    md += `### ⚡ Frontend Contract & Unit Tests (\`${vitestResults.totalPassed}\` passed, \`${vitestResults.totalFailed}\` failed)\n\n`;
    md += `| Test File | Status | Passed | Failed | Duration |\n`;
    md += `| :--- | :---: | :---: | :---: | :---: |\n`;
    for (const suite of vitestResults.suites) {
      const badge = suite.status === 'passed' ? '✅ Passed' : '❌ Failed';
      md += `| \`${suite.name}\` | ${badge} | ${suite.passed} | ${suite.failed} | ${suite.duration} |\n`;
    }
    md += `\n`;
  }

  return md;
}

const summary = generateMarkdownSummary();
const stepSummaryFile = process.env.GITHUB_STEP_SUMMARY;

if (stepSummaryFile) {
  fs.appendFileSync(stepSummaryFile, summary);
} else {
  console.log(summary);
}
