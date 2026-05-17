#import "updater.h"
#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <objc/message.h>

static WarplySparkleEventCallback event_callback = NULL;
static id updater_controller = nil;
static id updater_delegate = nil;
static BOOL updater_started = NO;

@interface WarplySparkleUpdaterDelegate : NSObject
@end

static void warply_sparkle_emit(int event, NSString *version, NSString *message) {
    if (event_callback == NULL) {
        return;
    }

    event_callback(
        event,
        version == nil ? NULL : [version UTF8String],
        message == nil ? NULL : [message UTF8String]
    );
}

static NSString *warply_sparkle_item_string(id item, NSString *key) {
    @try {
        id value = [item valueForKey:key];
        if ([value isKindOfClass:[NSString class]]) {
            return value;
        }
    } @catch (NSException *exception) {
        return nil;
    }
    return nil;
}

static BOOL warply_sparkle_error_is_benign(NSError *error) {
    if (error == nil) {
        return YES;
    }

    NSInteger code = [error code];
    return code == 1001 || code == 4007;
}

static BOOL warply_sparkle_load_framework(void) {
    if (NSClassFromString(@"SPUStandardUpdaterController") != Nil) {
        return YES;
    }

    NSURL *framework_url = [[[NSBundle mainBundle] privateFrameworksURL] URLByAppendingPathComponent:@"Sparkle.framework"];
    if (framework_url == nil) {
        return NO;
    }

    NSBundle *framework_bundle = [NSBundle bundleWithURL:framework_url];
    if (framework_bundle == nil) {
        return NO;
    }

    NSError *error = nil;
    if (![framework_bundle loadAndReturnError:&error]) {
        NSString *message = error == nil ? @"Sparkle.framework could not be loaded" : [error localizedDescription];
        warply_sparkle_emit(4, nil, message);
        return NO;
    }

    return YES;
}

static id warply_sparkle_updater(void) {
    if (updater_controller == nil) {
        return nil;
    }

    SEL updater_selector = NSSelectorFromString(@"updater");
    if (![updater_controller respondsToSelector:updater_selector]) {
        return nil;
    }

    return ((id (*)(id, SEL))objc_msgSend)(updater_controller, updater_selector);
}

void warply_sparkle_set_event_callback(WarplySparkleEventCallback callback) {
    event_callback = callback;
}

bool warply_sparkle_start(void) {
    if (updater_started) {
        return true;
    }

    if (![NSThread isMainThread]) {
        warply_sparkle_emit(4, nil, @"Sparkle updater must start on the main thread");
        return false;
    }

    if (!warply_sparkle_load_framework()) {
        warply_sparkle_emit(0, nil, nil);
        return false;
    }

    Class controller_class = NSClassFromString(@"SPUStandardUpdaterController");
    if (controller_class == Nil) {
        warply_sparkle_emit(4, nil, @"SPUStandardUpdaterController is unavailable");
        return false;
    }

    updater_delegate = [[WarplySparkleUpdaterDelegate alloc] init];
    SEL init_selector = NSSelectorFromString(@"initWithStartingUpdater:updaterDelegate:userDriverDelegate:");
    updater_controller = ((id (*)(id, SEL, BOOL, id, id))objc_msgSend)(
        [controller_class alloc],
        init_selector,
        YES,
        updater_delegate,
        nil
    );

    if (updater_controller == nil) {
        warply_sparkle_emit(4, nil, @"Sparkle updater controller could not be created");
        return false;
    }

    updater_started = YES;
    warply_sparkle_emit(1, nil, nil);
    return true;
}

bool warply_sparkle_check_for_update_information(void) {
    if (!warply_sparkle_start()) {
        return false;
    }

    id updater = warply_sparkle_updater();
    SEL selector = NSSelectorFromString(@"checkForUpdateInformation");
    if (updater == nil || ![updater respondsToSelector:selector]) {
        warply_sparkle_emit(4, nil, @"Sparkle update information check is unavailable");
        return false;
    }

    warply_sparkle_emit(3, nil, nil);
    ((void (*)(id, SEL))objc_msgSend)(updater, selector);
    return true;
}

bool warply_sparkle_check_for_updates(void) {
    if (!warply_sparkle_start()) {
        return false;
    }

    SEL selector = NSSelectorFromString(@"checkForUpdates:");
    if (updater_controller == nil || ![updater_controller respondsToSelector:selector]) {
        warply_sparkle_emit(4, nil, @"Sparkle update check is unavailable");
        return false;
    }

    ((void (*)(id, SEL, id))objc_msgSend)(updater_controller, selector, nil);
    return true;
}

@implementation WarplySparkleUpdaterDelegate

- (BOOL)updaterShouldPromptForPermissionToCheckForUpdates:(id)updater {
    return NO;
}

- (void)updater:(id)updater didFindValidUpdate:(id)item {
    NSString *version = warply_sparkle_item_string(item, @"displayVersionString");
    if (version == nil) {
        version = warply_sparkle_item_string(item, @"versionString");
    }
    warply_sparkle_emit(2, version, nil);
}

- (void)updaterDidNotFindUpdate:(id)updater {
    warply_sparkle_emit(1, nil, nil);
}

- (void)updaterDidNotFindUpdate:(id)updater error:(NSError *)error {
    warply_sparkle_emit(1, nil, nil);
}

- (void)updater:(id)updater didAbortWithError:(NSError *)error {
    if (warply_sparkle_error_is_benign(error)) {
        warply_sparkle_emit(1, nil, nil);
    } else {
        warply_sparkle_emit(4, nil, [error localizedDescription]);
    }
}

- (void)updater:(id)updater didFinishUpdateCycleForUpdateCheck:(NSInteger)updateCheck error:(NSError *)error {
    if (!warply_sparkle_error_is_benign(error)) {
        warply_sparkle_emit(4, nil, [error localizedDescription]);
    }
}

@end
