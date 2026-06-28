(function () {
  'use strict';

  function detectOS() {
    const ua = navigator.userAgent || '';
    if (/Mac/i.test(ua)) return 'macos';
    if (/Windows/i.test(ua)) return 'windows';
    if (/Linux/i.test(ua)) return 'linux';
    return null;
  }

  const PLATFORM_LABELS = {
    macos: 'Download for macOS',
    windows: 'Download for Windows',
    linux: 'Download for Linux'
  };

  const PLATFORM_HREFS = {
    macos: 'https://github.com/kulapoo/pomotoro/releases/latest',
    windows: 'https://github.com/kulapoo/pomotoro/releases/latest',
    linux: 'https://github.com/kulapoo/pomotoro/releases/latest'
  };

  function initDownloadButton() {
    const btn = document.getElementById('download-btn');
    if (!btn) return;
    const os = detectOS();
    if (os && PLATFORM_LABELS[os]) {
      btn.textContent = PLATFORM_LABELS[os];
      btn.href = PLATFORM_HREFS[os];
    }
  }

  function initOtherPlatforms() {
    const toggle = document.getElementById('other-platforms-toggle');
    const panel = document.getElementById('other-platforms');
    if (!toggle || !panel) return;

    toggle.addEventListener('click', function () {
      const isOpen = panel.classList.toggle('is-expanded');
      toggle.setAttribute('aria-expanded', String(isOpen));
    });
  }

  function initReveal() {
    const prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const items = document.querySelectorAll('.reveal');

    if (prefersReduced || !('IntersectionObserver' in window)) {
      items.forEach(function (el) { el.classList.add('is-visible'); });
      return;
    }

    const observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add('is-visible');
          observer.unobserve(entry.target);
        }
      });
    }, { rootMargin: '0px 0px -10% 0px', threshold: 0.1 });

    items.forEach(function (el) { observer.observe(el); });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function () {
      initDownloadButton();
      initOtherPlatforms();
      initReveal();
    });
  } else {
    initDownloadButton();
    initOtherPlatforms();
    initReveal();
  }
})();
