/*
   raw-window-handle gives us an NSView*, but nativefiledialog-extended wants the NSWindow*
   that contains it. Must be called on the main thread (the Rust side checks this first).
*/

#import <AppKit/AppKit.h>

void* nfd_rs_window_of_view(void* view) {
    if (view == NULL) {
        return NULL;
    }
    return (void*)[(NSView*)view window];
}
