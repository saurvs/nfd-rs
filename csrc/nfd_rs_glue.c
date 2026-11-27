/*
   Build-time checks that src/ffi.rs matches the bundled nfd.h.

   The table below is only referenced by the ffi layout unit test, so it is not linked into
   normal builds.
*/

#include <stddef.h>

#include "nfd.h"

#if NFD_INTERFACE_VERSION != 2
#error "nfd-rs: src/ffi.rs describes NFD_INTERFACE_VERSION 2; update the bindings for the new nfd.h"
#endif

const size_t nfd_rs_layout[] = {
    NFD_INTERFACE_VERSION,
    sizeof(nfdresult_t),
    sizeof(nfdpathsetsize_t),
    sizeof(nfdfiltersize_t),
    sizeof(nfdu8filteritem_t),
    offsetof(nfdu8filteritem_t, spec),
    sizeof(nfdwindowhandle_t),
    offsetof(nfdwindowhandle_t, handle),
    sizeof(nfdopendialogu8args_t),
    offsetof(nfdopendialogu8args_t, filterCount),
    offsetof(nfdopendialogu8args_t, defaultPath),
    offsetof(nfdopendialogu8args_t, parentWindow),
    offsetof(nfdopendialogu8args_t, title),
    offsetof(nfdopendialogu8args_t, acceptLabel),
    offsetof(nfdopendialogu8args_t, cancelLabel),
    sizeof(nfdsavedialogu8args_t),
    offsetof(nfdsavedialogu8args_t, filterCount),
    offsetof(nfdsavedialogu8args_t, defaultPath),
    offsetof(nfdsavedialogu8args_t, defaultName),
    offsetof(nfdsavedialogu8args_t, parentWindow),
    offsetof(nfdsavedialogu8args_t, title),
    offsetof(nfdsavedialogu8args_t, acceptLabel),
    offsetof(nfdsavedialogu8args_t, cancelLabel),
    sizeof(nfdpickfolderu8args_t),
    offsetof(nfdpickfolderu8args_t, parentWindow),
    offsetof(nfdpickfolderu8args_t, title),
    offsetof(nfdpickfolderu8args_t, acceptLabel),
    offsetof(nfdpickfolderu8args_t, cancelLabel),
    NFD_ERROR * 100 + NFD_OKAY * 10 + NFD_CANCEL,
};

const size_t nfd_rs_layout_len = sizeof(nfd_rs_layout) / sizeof(nfd_rs_layout[0]);
