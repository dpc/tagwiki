/* https://raw.githubusercontent.com/ccampbell/mousetrap/master/plugins/global-bind/mousetrap-global-bind.min.js */
(function(a){var c={},d=a.prototype.stopCallback;a.prototype.stopCallback=function(e,b,a,f){return this.paused?!0:c[a]||c[f]?!1:d.call(this,e,b,a)};a.prototype.bindGlobal=function(a,b,d){this.bind(a,b,d);if(a instanceof Array)for(b=0;b<a.length;b++)c[a[b]]=!0;else c[a]=!0};a.init()})(Mousetrap);

// OWN, CUSTOM CODE BELOW

// for some odd reason, without using a timeout, `.click()` does not always work
function reliableClick(domElement) {
  window.setTimeout(function () {
    domElement.click();
  }, 1);
}

function reliableClickById(id) {
  const elem = document.getElementById(id);
  if (elem) {
    reliableClick(elem);
  }
}

function focusById(id) {
  const elem = document.getElementById(id);
  if (elem) {
    elem.focus();
  }
}

function focusAppendToById(id) {
  const elem = document.getElementById(id);
  if (elem) {
    elem.focus();
    elem.setSelectionRange(elem.value.length, elem.value.length);
  }
}

function focusPrependToById(id) {
  const elem = document.getElementById(id);
  if (elem) {
    elem.focus();
    elem.setSelectionRange(0, 0);
  }
}
function smartFocusEditById(id) {
  const elem = document.getElementById(id);
  if (elem) {
    if (elem.classList.contains("prepend")) {
      focusPrependToById(id);
    } else {
      focusAppendToById(id);
    }
  }
}

// Modified: https://stackoverflow.com/a/35173443/12271202
// dir: 1 for down, -1 for up
function indexFocusSwitch(dir) {
    const parentArea = document.getElementById('page-content');
    if (parentArea) {
        var focussableElements = 'a:not([disabled])';
        var focussable = Array.prototype.filter.call(parentArea.querySelectorAll(focussableElements),
          function (element) {
              return true;
          }
        );
        var index = focussable.indexOf(document.activeElement);
        if(focussable.length > 0) {
           var nextElement = focussable[(index + dir + focussable.length) % focussable.length] || focussable[0];
           nextElement.focus();
        }                    
    }
}

Mousetrap.bind('n', function() {
  // window.location.href = '?edit=true';
  reliableClickById('new-button');
});

Mousetrap.bind('e', function() {
  reliableClickById('edit-button');
});

Mousetrap.bind('d', function() {
  reliableClickById('delete-button');
});

Mousetrap.bindGlobal(['alt+enter', 'ctrl+enter'], function() {
  reliableClickById('save-button');
});

Mousetrap.bind('s', function() {
  reliableClickById('save-button');
});


Mousetrap.bindGlobal('esc', function() {
  let textarea = document.getElementById('source-editor');
  let queryText = document.getElementById('query-text');
  if (textarea && textarea === document.activeElement) {
    textarea.blur();
    let button = document.getElementById('cancel-button');
    if (button) {
      button.focus();
    }
  } else if (queryText && queryText === document.activeElement) {
    queryText.blur();
  } else {
    reliableClickById('cancel-button');
    reliableClickById('up-button');
  }
});

Mousetrap.bind(['ctrl+left', 'h'], function() {
  reliableClickById('cancel-button');
  reliableClickById('up-button');
});

Mousetrap.bind(['ctrl+right', 'l'], function() {
  const elem = document.activeElement;
  if (elem) {
    reliableClick(elem);
  }
});

Mousetrap.bind(['ctrl+down', 'j'], function() {
  indexFocusSwitch(1);
});

Mousetrap.bind(['ctrl+up', 'k'], function() {
  indexFocusSwitch(-1);
});

Mousetrap.bind(['/'], function() {
  focusAppendToById('query-text');
});

Mousetrap.bindGlobal(['alt+/', 'alt+f', 'alt+q'], function() {
  focusAppendToById('query-text');
});

// auto-select first element on index pages
indexFocusSwitch(1);

// if we're editing a page, let's start at the the right placeA
smartFocusEditById('source-editor');
