(function () {
  'use strict';

  const STORAGE_LANG = 'shredlink_lang';
  const STORAGE_DARK = 'shredlink_dark';

  const t = {
    en: {
      title: 'ShredLink',
      tagline: ' ',
      contentLabel: 'Content',
      contentPlaceholder: 'Paste or type the text you want to share securely…',
      contentHint: 'Optional: protect with a password. Leave empty for a public link.',
      passwordLabel: 'Password (optional)',
      expireLabel: 'Expire after',
      expire10m: '10 minutes',
      expire30m: '30 minutes',
      expire1h: '1 hour',
      expire2h: '2 hours',
      expire3h: '3 hours',
      expire1d: '1 day',
      oneTimeView: 'One-time view (link invalid after first open)',
      oneTimePassword: 'One-time password (expires after correct password once)',
      createBtn: 'Create link',
      yourLink: 'Your secure link:',
      copy: 'Copy',
      copied: 'Copied!',
      createAnother: 'Create another link',
      footer: 'Content is encrypted and can be set to expire or become invalid after one view.',
      errorRequired: 'Please enter some text.',
      errorNetwork: 'Network error. Check your connection and try again.',
      errorGeneric: 'Failed to create link. Try again.',
      copyManual: 'Select and copy manually',
    },
    fa: {
      title: 'ShredLink',
      tagline: ' ',
      contentLabel: 'محتوا',
      contentPlaceholder: 'متن را اینجا بچسبانید یا تایپ کنید…',
      contentHint: 'اختیاری: با رمز عبور محافظت کنید. برای لینک عمومی خالی بگذارید.',
      passwordLabel: 'رمز عبور (اختیاری)',
      expireLabel: 'انقضا پس از',
      expire10m: '۱۰ دقیقه',
      expire30m: '۳۰ دقیقه',
      expire1h: '۱ ساعت',
      expire2h: '۲ ساعت',
      expire3h: '۳ ساعت',
      expire1d: '۱ روز',
      oneTimeView: 'یک‌بار مشاهده (لینک بعد از اولین باز شدن غیرفعال می‌شود)',
      oneTimePassword: 'یک‌بار رمز (بعد از یک بار وارد کردن صحیح رمز، لینک منقضی می‌شود)',
      createBtn: 'ساخت لینک',
      yourLink: 'لینک امن شما:',
      copy: 'کپی',
      copied: 'کپی شد!',
      createAnother: 'ساخت لینک دیگر',
      footer: 'محتوا رمزنگاری شده و قابل انقضا یا یک‌بار مصرف است.',
      errorRequired: 'لطفاً متنی وارد کنید.',
      errorNetwork: 'خطای شبکه. اتصال را بررسی کنید.',
      errorGeneric: 'ساخت لینک ناموفق بود. دوباره تلاش کنید.',
      copyManual: 'انتخاب و کپی دستی',
    },
  };

  let currentLang = localStorage.getItem(STORAGE_LANG) || 'en';
  if (currentLang !== 'en' && currentLang !== 'fa') currentLang = 'en';

  function applyLang() {
    document.documentElement.lang = currentLang === 'fa' ? 'fa' : 'en';
    document.documentElement.dir = currentLang === 'fa' ? 'rtl' : 'ltr';
    document.body.classList.toggle('lang-fa', currentLang === 'fa');

    document.querySelectorAll('[data-i18n]').forEach(function (el) {
      var key = el.getAttribute('data-i18n');
      if (t[currentLang][key]) el.textContent = t[currentLang][key];
    });
    document.querySelectorAll('[data-i18n-placeholder]').forEach(function (el) {
      var key = el.getAttribute('data-i18n-placeholder');
      if (t[currentLang][key]) el.placeholder = t[currentLang][key];
    });
    document.querySelectorAll('select#expire option').forEach(function (opt) {
      var key = opt.getAttribute('data-i18n');
      if (key && t[currentLang][key]) opt.textContent = t[currentLang][key];
    });

    document.getElementById('lang-en').classList.toggle('font-medium', currentLang === 'en');
    document.getElementById('lang-fa').classList.toggle('font-medium', currentLang === 'fa');
  }

  function initDark() {
    var stored = localStorage.getItem(STORAGE_DARK);
    var dark = stored === '1' || (!stored && window.matchMedia('(prefers-color-scheme: dark)').matches);
    if (dark) document.documentElement.classList.add('dark');
    else document.documentElement.classList.remove('dark');
  }

  function toggleDark() {
    document.documentElement.classList.toggle('dark');
    localStorage.setItem(STORAGE_DARK, document.documentElement.classList.contains('dark') ? '1' : '0');
  }

  window.i18n = {
    lang: function () { return currentLang; },
    t: function (key) { return (t[currentLang] && t[currentLang][key]) ? t[currentLang][key] : key; },
    setLang: function (lang) {
      if (lang !== 'en' && lang !== 'fa') return;
      currentLang = lang;
      localStorage.setItem(STORAGE_LANG, currentLang);
      applyLang();
    },
    applyLang: applyLang,
    initDark: initDark,
    toggleDark: toggleDark,
  };

  initDark();
  applyLang();

  document.getElementById('dark-toggle').addEventListener('click', toggleDark);
  document.getElementById('lang-en').addEventListener('click', function () { window.i18n.setLang('en'); });
  document.getElementById('lang-fa').addEventListener('click', function () { window.i18n.setLang('fa'); });
})();
