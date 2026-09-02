import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import QuickLookModal from './QuickLookModal.svelte';
import type { QuickLookAttachment } from './QuickLookModal.svelte';

// Mock pdfjs-dist
vi.mock('pdfjs-dist', () => {
  return {
    GlobalWorkerOptions: { workerSrc: '' },
    version: '4.0.0',
    getDocument: vi.fn(() => ({
      promise: Promise.resolve({
        numPages: 3,
        getPage: vi.fn((pageNo: number) =>
          Promise.resolve({
            getViewport: vi.fn(() => ({ width: 600, height: 800 })),
            render: vi.fn(() => ({
              promise: Promise.resolve(),
              cancel: vi.fn()
            }))
          })
        ),
        destroy: vi.fn()
      })
    }))
  };
});

describe('QuickLookModal Component', () => {
  let revokeSpy: ReturnType<typeof vi.spyOn>;
  let createSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    revokeSpy = vi.spyOn(window.URL, 'revokeObjectURL');
    createSpy = vi.spyOn(window.URL, 'createObjectURL').mockReturnValue('blob:test-image-url');
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('does not render content when isOpen is false', () => {
    const attachment: QuickLookAttachment = {
      filename: 'sample.png',
      size: 1024,
      contentType: 'image/png'
    };

    const { queryByRole } = render(QuickLookModal, {
      isOpen: false,
      attachment
    });

    expect(queryByRole('dialog')).toBeNull();
  });

  it('renders image preview and handles zoom in/out/reset and double click', async () => {
    const attachment: QuickLookAttachment = {
      filename: 'sample.png',
      size: 1024,
      contentType: 'image/png',
      data: new Uint8Array([1, 2, 3])
    };

    const { getByRole, getByAltText, getByTitle, getByText } = render(QuickLookModal, {
      isOpen: true,
      attachment
    });

    expect(getByRole('dialog')).toBeTruthy();
    const img = getByAltText('sample.png') as HTMLImageElement;
    expect(img).toBeTruthy();
    expect(img.src).toContain('blob:test-image-url');

    // Default zoom is 100%
    expect(getByText('100%')).toBeTruthy();

    // Zoom in
    const zoomInBtn = getByTitle('Zoom in');
    await fireEvent.click(zoomInBtn);
    expect(getByText('125%')).toBeTruthy();

    // Zoom out
    const zoomOutBtn = getByTitle('Zoom out');
    await fireEvent.click(zoomOutBtn);
    expect(getByText('100%')).toBeTruthy();

    // Double-click image toggles zoom to 175%
    await fireEvent.doubleClick(img);
    expect(getByText('175%')).toBeTruthy();

    // Reset zoom
    const resetBtn = getByTitle('Reset Zoom');
    await fireEvent.click(resetBtn);
    expect(getByText('100%')).toBeTruthy();
  });

  it('renders PDF preview and supports page navigation controls', async () => {
    const attachment: QuickLookAttachment = {
      filename: 'document.pdf',
      size: 512000,
      contentType: 'application/pdf',
      data: new Uint8Array([0x25, 0x50, 0x44, 0x46]).buffer
    };

    const { getByText, getByTitle, findByText } = render(QuickLookModal, {
      isOpen: true,
      attachment
    });

    // Wait for pdfjs getDocument promise to resolve and page info to display
    const pageText = await findByText('/ 3');
    expect(pageText).toBeTruthy();

    const nextBtn = getByTitle('Next Page');
    await fireEvent.click(nextBtn);

    const prevBtn = getByTitle('Previous Page');
    await fireEvent.click(prevBtn);
  });

  it('renders fallback card for non-previewable files and handles download button', async () => {
    const onDownload = vi.fn();
    const attachment: QuickLookAttachment = {
      filename: 'archive.zip',
      size: 1048576,
      contentType: 'application/zip'
    };

    const { getByText, getAllByText } = render(QuickLookModal, {
      isOpen: true,
      attachment,
      onDownload
    });

    expect(getAllByText('archive.zip').length).toBeGreaterThan(0);
    expect(getByText(/No live preview available for this file type/)).toBeTruthy();

    // Download button in fallback card
    const downloadBtn = getByText('Download File');
    await fireEvent.click(downloadBtn);

    expect(onDownload).toHaveBeenCalledWith(attachment);
  });

  it('closes modal on Escape key press and backdrop click', async () => {
    const onClose = vi.fn();
    const attachment: QuickLookAttachment = {
      filename: 'photo.jpg',
      size: 2048,
      contentType: 'image/jpeg',
      data: new Uint8Array([255, 216, 255])
    };

    const { getByRole } = render(QuickLookModal, {
      isOpen: true,
      attachment,
      onClose
    });

    // Fire Escape key event on window
    await fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();

    // Fire backdrop click
    const dialog = getByRole('dialog');
    await fireEvent.click(dialog);
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('revokes object URLs on dismissal / unmount', async () => {
    const attachment: QuickLookAttachment = {
      filename: 'photo.jpg',
      size: 2048,
      contentType: 'image/jpeg',
      data: new Uint8Array([255, 216, 255])
    };

    const { unmount } = render(QuickLookModal, {
      isOpen: true,
      attachment
    });

    expect(createSpy).toHaveBeenCalled();

    unmount();

    expect(revokeSpy).toHaveBeenCalledWith('blob:test-image-url');
  });
});
