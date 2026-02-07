(function () {
  'use strict';

  const form = document.getElementById('create-form');
  const createSection = document.getElementById('create-section');
  const resultSection = document.getElementById('result-section');
  const resultUrl = document.getElementById('result-url');
  const copyBtn = document.getElementById('copy-btn');
  const newLinkBtn = document.getElementById('new-link-btn');
  const formError = document.getElementById('form-error');
  const formSuccess = document.getElementById('form-success');
  const submitBtn = document.getElementById('submit-btn');

  function showError(msg) {
    formSuccess.classList.add('hidden');
    formError.textContent = msg;
    formError.classList.remove('hidden');
  }

  function showSuccess(msg) {
    formError.classList.add('hidden');
    formSuccess.textContent = msg;
    formSuccess.classList.remove('hidden');
  }

  function hideMessages() {
    formError.classList.add('hidden');
    formSuccess.classList.add('hidden');
  }

  form.addEventListener('submit', async function (e) {
    e.preventDefault();
    hideMessages();
    submitBtn.disabled = true;

    var text = document.getElementById('text').value.trim();
    if (!text) {
      showError(window.i18n ? window.i18n.t('errorRequired') : 'Please enter some text.');
      submitBtn.disabled = false;
      return;
    }

    var password = document.getElementById('password').value;
    var expireMinutes = parseInt(document.getElementById('expire').value, 10) || 0;
    var oneTimeView = document.getElementById('one_time_view').checked;
    var oneTimePassword = document.getElementById('one_time_password').checked;

    var body = {
      text: text,
      password: password || null,
      expire_minutes: expireMinutes > 0 ? expireMinutes : null,
      expire_hours: null,
      one_time_view: oneTimeView,
      one_time_password: oneTimePassword,
    };

    try {
      var res = await fetch('/api/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      var data = await res.json().catch(function () { return {}; });

      if (!res.ok) {
        showError(data.error || (window.i18n ? window.i18n.t('errorGeneric') : 'Failed to create link. Try again.'));
        submitBtn.disabled = false;
        return;
      }

      resultUrl.value = data.url || '';
      createSection.classList.add('hidden');
      resultSection.classList.remove('hidden');
    } catch (err) {
      showError(window.i18n ? window.i18n.t('errorNetwork') : 'Network error. Check your connection and try again.');
    }
    submitBtn.disabled = false;
  });

  copyBtn.addEventListener('click', function () {
    resultUrl.select();
    resultUrl.setSelectionRange(0, 99999);
    try {
      navigator.clipboard.writeText(resultUrl.value);
      copyBtn.textContent = window.i18n ? window.i18n.t('copied') : 'Copied!';
      setTimeout(function () {
        copyBtn.textContent = window.i18n ? window.i18n.t('copy') : 'Copy';
      }, 2000);
    } catch (_) {
      copyBtn.textContent = window.i18n ? window.i18n.t('copyManual') : 'Select and copy manually';
      setTimeout(function () {
        copyBtn.textContent = window.i18n ? window.i18n.t('copy') : 'Copy';
      }, 2000);
    }
  });

  newLinkBtn.addEventListener('click', function () {
    resultSection.classList.add('hidden');
    createSection.classList.remove('hidden');
    form.reset();
    document.getElementById('expire').selectedIndex = 0;
    hideMessages();
  });
})();
