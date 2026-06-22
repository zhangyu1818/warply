#import <AppKit/AppKit.h>

NSString *get_default_app_bundle_for_file(NSString *file_path) {
    NSURL *fileUrl = [NSURL fileURLWithPath:file_path];
    NSURL *appUrl = [[NSWorkspace sharedWorkspace] URLForApplicationToOpenURL:fileUrl];
    if (!appUrl) {
        return nil;
    }

    NSBundle *appBundle = [NSBundle bundleWithURL:appUrl];
    if (!appBundle) {
        return nil;
    }
    return [appBundle bundleIdentifier];
}

static NSString *editor_app_display_name(NSBundle *bundle, NSURL *appURL) {
    NSString *name = [[bundle localizedInfoDictionary] objectForKey:@"CFBundleDisplayName"];
    if (!name) {
        name = [[bundle infoDictionary] objectForKey:@"CFBundleDisplayName"];
    }
    if (!name) {
        name = [[bundle infoDictionary] objectForKey:@"CFBundleName"];
    }
    if (!name) {
        name = [[appURL URLByDeletingPathExtension] lastPathComponent];
    }
    return name;
}

static NSString *editor_app_icon_path(NSWorkspace *workspace, NSURL *appURL, NSString *iconCacheDirectory, NSString *bundleIdentifier) {
    if (!iconCacheDirectory || [iconCacheDirectory length] == 0) {
        return nil;
    }

    NSCharacterSet *unsafeCharacters = [[NSCharacterSet alphanumericCharacterSet] invertedSet];
    NSString *safeName = [[bundleIdentifier componentsSeparatedByCharactersInSet:unsafeCharacters] componentsJoinedByString:@"_"];
    NSString *iconPath = [iconCacheDirectory stringByAppendingPathComponent:[safeName stringByAppendingPathExtension:@"png"]];
    NSFileManager *fileManager = [NSFileManager defaultManager];
    if ([fileManager fileExistsAtPath:iconPath]) {
        return iconPath;
    }

    if (![fileManager createDirectoryAtPath:iconCacheDirectory withIntermediateDirectories:YES attributes:nil error:nil]) {
        return nil;
    }

    NSImage *icon = [workspace iconForFile:[appURL path]];
    if (!icon) {
        return nil;
    }

    NSImage *iconCopy = [icon copy];
    [iconCopy setSize:NSMakeSize(32.0, 32.0)];
    CGImageRef cgImage = [iconCopy CGImageForProposedRect:NULL context:nil hints:nil];
    NSBitmapImageRep *bitmapRep = nil;
    if (cgImage) {
        bitmapRep = [[NSBitmapImageRep alloc] initWithCGImage:cgImage];
    } else {
        NSData *tiffData = [iconCopy TIFFRepresentation];
        if (tiffData) {
            bitmapRep = [[NSBitmapImageRep alloc] initWithData:tiffData];
        }
    }

    NSData *pngData = [bitmapRep representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
    BOOL wroteIcon = pngData && [pngData writeToFile:iconPath atomically:YES];

    [bitmapRep release];
    [iconCopy release];

    return wroteIcon ? iconPath : nil;
}

NSString *scan_editor_apps_json(NSString *iconCacheDirectory) {
    NSArray<NSString *> *extensions = @[@"txt", @"md", @"swift", @"js", @"ts", @"json", @"py", @"html", @"css", @"yml"];
    NSWorkspace *workspace = [NSWorkspace sharedWorkspace];
    NSMutableSet<NSString *> *seenBundleIdentifiers = [NSMutableSet set];
    NSMutableArray<NSDictionary *> *apps = [NSMutableArray array];

    for (NSString *ext in extensions) {
        NSString *probeName = [@"portal-editor-probe" stringByAppendingPathExtension:ext];
        NSString *probePath = [NSTemporaryDirectory() stringByAppendingPathComponent:probeName];
        NSURL *probeURL = [NSURL fileURLWithPath:probePath];
        NSArray<NSURL *> *appURLs = nil;

        if (@available(macOS 12.0, *)) {
            appURLs = [workspace URLsForApplicationsToOpenURL:probeURL];
        } else {
            NSURL *defaultAppURL = [workspace URLForApplicationToOpenURL:probeURL];
            appURLs = defaultAppURL ? @[defaultAppURL] : @[];
        }

        for (NSURL *appURL in appURLs) {
            NSBundle *bundle = [NSBundle bundleWithURL:appURL];
            NSString *bundleIdentifier = [bundle bundleIdentifier];
            if (!bundle || !bundleIdentifier || [seenBundleIdentifiers containsObject:bundleIdentifier]) {
                continue;
            }

            [seenBundleIdentifiers addObject:bundleIdentifier];
            NSString *displayName = editor_app_display_name(bundle, appURL);
            NSString *iconPath = editor_app_icon_path(workspace, appURL, iconCacheDirectory, bundleIdentifier);

            NSMutableDictionary *app = [NSMutableDictionary dictionary];
            [app setObject:bundleIdentifier forKey:@"bundle_identifier"];
            [app setObject:displayName forKey:@"display_name"];
            [app setObject:[appURL path] forKey:@"bundle_url"];
            if (iconPath) {
                [app setObject:iconPath forKey:@"icon_path"];
            }
            [apps addObject:app];
        }
    }

    NSData *jsonData = [NSJSONSerialization dataWithJSONObject:apps options:0 error:nil];
    if (!jsonData) {
        return @"[]";
    }

    NSString *json = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
    return [json autorelease];
}
