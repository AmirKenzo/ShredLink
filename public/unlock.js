(function () {
  'use strict';

  var params = new URLSearchParams(window.location.search);
  var token = params.get('token') || '';

  var msgs = {
    en: {
      title: 'This link is protected',
      desc: 'Enter the password to view the content.',
      back: 'Back home',
      unlock: 'Unlock',
      wrong: 'Wrong password',
      network: 'Network error',
      placeholder: 'Password',
      copyAll: 'Copy all',
      copied: 'Copied!'
    },
    fa: {
      title: 'این لینک محافظت شده است',
      desc: 'رمز عبور را برای مشاهده محتوا وارد کنید.',
      back: 'بازگشت به خانه',
      unlock: 'باز کردن',
      wrong: 'رمز اشتباه',
      network: 'خطای شبکه',
      placeholder: 'رمز عبور',
      copyAll: 'کپی همه',
      copied: 'کپی شد!'
    }
  };

  function lang() {
    return document.documentElement.getAttribute('data-lang') || 'en';
  }

  function applyLang(l) {
    var m = msgs[l] || msgs.en;
    var t = document.querySelector('[data-msg="title"]');
    if (t) t.textContent = m.title;
    var d = document.querySelector('[data-msg="desc"]');
    if (d) d.textContent = m.desc;
    var b = document.querySelector('[data-msg="back"]');
    if (b) b.textContent = m.back;
    var u = document.querySelector('[data-msg="unlock"]');
    if (u) u.textContent = m.unlock;
    var p = document.getElementById('pass-input');
    if (p) p.placeholder = m.placeholder;
    document.querySelectorAll('.lang-btn').forEach(function (el) {
      el.classList.toggle('font-medium', el.id === 'lang-' + (l === 'fa' ? 'fa' : 'en'));
    });
  }

  function initUi() {
    var l = lang();
    document.body.classList.toggle('lang-fa', l === 'fa');
    applyLang(l);

    var darkToggle = document.getElementById('dark-toggle');
    if (darkToggle) {
      darkToggle.addEventListener('click', function () {
        document.documentElement.classList.toggle('dark');
        localStorage.setItem('shredlink_dark', document.documentElement.classList.contains('dark') ? '1' : '0');
      });
    }

    var langEn = document.getElementById('lang-en');
    if (langEn) {
      langEn.addEventListener('click', function () {
        localStorage.setItem('shredlink_lang', 'en');
        document.documentElement.lang = 'en';
        document.documentElement.dir = 'ltr';
        document.documentElement.setAttribute('data-lang', 'en');
        document.body.classList.remove('lang-fa');
        applyLang('en');
      });
    }

    var langFa = document.getElementById('lang-fa');
    if (langFa) {
      langFa.addEventListener('click', function () {
        localStorage.setItem('shredlink_lang', 'fa');
        document.documentElement.lang = 'fa';
        document.documentElement.dir = 'rtl';
        document.documentElement.setAttribute('data-lang', 'fa');
        document.body.classList.add('lang-fa');
        applyLang('fa');
      });
    }
  }

  initUi();

  var form = document.getElementById('unlock');
  if (!form) return;

  form.addEventListener('submit', function (e) {
    e.preventDefault();
    var password = form.password.value;
    var errEl = document.getElementById('err');
    errEl.classList.add('hidden');

    var url = '/api/unlock/' + encodeURIComponent(token);
    fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ password: password })
    })
      .then(function (r) {
        return r.json().then(function (data) {
          if (r.ok) {
            var raw = data.text || '';
            var esc = function (s) {
              return String(s)
                .replace(/&/g, '&amp;')
                .replace(/</g, '&lt;')
                .replace(/>/g, '&gt;')
                .replace(/"/g, '&quot;');
            };
            var payload = JSON.stringify(raw).replace(/<\/script>/gi, '<\\/script>');
            var l = lang();
            var m = msgs[l] || msgs.en;
            var html =
              '<div class="max-w-3xl mx-auto px-4 py-8">' +
              '<header class="flex items-center justify-between mb-6">' +
              '<a href="/" class="text-indigo-600 dark:text-indigo-400 hover:underline font-medium">' + m.back + '</a>' +
              '<div class="flex items-center gap-3">' +
              '<button type="button" id="dark-toggle" class="p-2 rounded-lg text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-700">' +
              '<svg id="icon-sun" class="w-5 h-5 hidden dark:block" fill="currentColor" viewBox="0 0 20 20"><path d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"/></svg>' +
              '<svg id="icon-moon" class="w-5 h-5 block dark:hidden" fill="currentColor" viewBox="0 0 20 20"><path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"/></svg>' +
              '</button>' +
              '<div class="lang-switcher inline-flex rounded-xl border border-slate-200 dark:border-slate-600 overflow-hidden bg-slate-100/80 dark:bg-slate-700/50"><button type="button" id="lang-en" class="lang-btn min-h-[44px] min-w-[52px] px-4 py-2.5 text-sm transition-colors text-slate-500 dark:text-slate-400 hover:bg-slate-200/70 dark:hover:bg-slate-600/50' + (l === 'en' ? ' font-medium' : '') + '">EN</button><button type="button" id="lang-fa" class="lang-btn min-h-[44px] min-w-[52px] px-4 py-2.5 text-sm transition-colors text-slate-500 dark:text-slate-400 hover:bg-slate-200/70 dark:hover:bg-slate-600/50' + (l === 'fa' ? ' font-medium' : '') + '">فا</button></div>' +
              '</div></header>' +
              '<main class="bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-slate-200 dark:border-slate-700 overflow-hidden">' +
              '<div class="p-6 sm:p-8"><pre id="content-display" dir="auto" class="content-display whitespace-pre-wrap break-words text-slate-800 dark:text-slate-100 text-sm leading-relaxed max-h-[70vh] overflow-y-auto" style="unicode-bidi:plaintext;text-align:start">' + esc(raw) + '</pre></div>' +
              '<div class="px-6 sm:px-8 pb-6 flex flex-wrap gap-3 border-t border-slate-200 dark:border-slate-700 pt-4">' +
              '<button type="button" id="copy-all" class="px-4 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl font-medium text-sm">' + m.copyAll + '</button>' +
              '<a href="/" class="text-sm text-slate-600 dark:text-slate-400 hover:underline">' + m.back + '</a>' +
              '</div></main></div>';

            document.body.innerHTML = html;
            document.body.className = 'bg-slate-50 dark:bg-slate-900 min-h-screen text-slate-800 dark:text-slate-200 antialiased';

            var copyBtn = document.getElementById('copy-all');
            if (copyBtn) {
              var textToCopy = raw;
              function fallbackCopy() {
                var ta = document.createElement('textarea');
                ta.value = textToCopy;
                ta.style.position = 'fixed';
                ta.style.left = '-9999px';
                document.body.appendChild(ta);
                ta.select();
                try { document.execCommand('copy'); } catch (e) {}
                document.body.removeChild(ta);
              }
              function showCopied() {
                copyBtn.textContent = (document.documentElement.getAttribute('data-lang') === 'fa' ? msgs.fa.copied : msgs.en.copied);
                setTimeout(function () {
                  copyBtn.textContent = document.documentElement.getAttribute('data-lang') === 'fa' ? msgs.fa.copyAll : msgs.en.copyAll;
                }, 2000);
              }
              copyBtn.addEventListener('click', function () {
                if (navigator.clipboard && navigator.clipboard.writeText) {
                  navigator.clipboard.writeText(textToCopy).then(showCopied, function () { fallbackCopy(); showCopied(); });
                } else {
                  fallbackCopy();
                  showCopied();
                }
              });
            }

            var newDark = document.getElementById('dark-toggle');
            if (newDark) {
              newDark.addEventListener('click', function () {
                document.documentElement.classList.toggle('dark');
                localStorage.setItem('shredlink_dark', document.documentElement.classList.contains('dark') ? '1' : '0');
              });
            }
            var newEn = document.getElementById('lang-en');
            var newFa = document.getElementById('lang-fa');
            if (newEn) newEn.addEventListener('click', function () { localStorage.setItem('shredlink_lang', 'en'); location.reload(); });
            if (newFa) newFa.addEventListener('click', function () { localStorage.setItem('shredlink_lang', 'fa'); location.reload(); });
            var cur = document.documentElement.getAttribute('data-lang') || 'en';
            document.querySelectorAll('.lang-btn').forEach(function (el) {
              el.classList.toggle('font-medium', el.id === 'lang-' + cur);
            });
          } else {
            errEl.textContent = (lang() === 'fa' ? msgs.fa.wrong : (data.error || msgs.en.wrong));
            errEl.classList.remove('hidden');
          }
        });
      })
      .catch(function () {
        errEl.textContent = (lang() === 'fa' ? msgs.fa.network : msgs.en.network);
        errEl.classList.remove('hidden');
      });
  });
})();
