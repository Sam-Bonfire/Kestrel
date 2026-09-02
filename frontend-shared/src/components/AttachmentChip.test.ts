import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import AttachmentChip from './AttachmentChip.svelte';

describe('AttachmentChip', () => {
  const attachment = {
    filename: 'test-document.pdf',
    size: 204800,
    contentType: 'application/pdf'
  };

  it('renders filename and formatted byte size', () => {
    const { getByText } = render(AttachmentChip, {
      attachment,
    });

    expect(getByText('test-document.pdf')).toBeTruthy();
    expect(getByText('200 KB')).toBeTruthy();
  });

  it('triggers onPreview when preview button is clicked', async () => {
    const onPreview = vi.fn();
    const onDownload = vi.fn();

    const { getByTitle } = render(AttachmentChip, {
      attachment,
      onPreview,
      onDownload
    });

    const previewBtn = getByTitle('Preview');
    await fireEvent.click(previewBtn);

    expect(onPreview).toHaveBeenCalledTimes(1);
    expect(onPreview).toHaveBeenCalledWith(attachment);
    expect(onDownload).not.toHaveBeenCalled();
  });

  it('triggers onDownload when download button is clicked', async () => {
    const onPreview = vi.fn();
    const onDownload = vi.fn();

    const { getByTitle } = render(AttachmentChip, {
      attachment,
      onPreview,
      onDownload
    });

    const downloadBtn = getByTitle('Download');
    await fireEvent.click(downloadBtn);

    expect(onDownload).toHaveBeenCalledTimes(1);
    expect(onDownload).toHaveBeenCalledWith(attachment);
    expect(onPreview).not.toHaveBeenCalled();
  });

  it('hides preview button for non-previewable files', () => {
    const zipAttachment = {
      filename: 'archive.zip',
      size: 1048576,
      contentType: 'application/zip'
    };

    const { queryByTitle, getByTitle } = render(AttachmentChip, {
      attachment: zipAttachment,
      onPreview: vi.fn(),
      onDownload: vi.fn()
    });

    expect(queryByTitle('Preview')).toBeNull();
    expect(getByTitle('Download')).toBeTruthy();
  });
});
