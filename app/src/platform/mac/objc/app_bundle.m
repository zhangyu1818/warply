#import <AppKit/AppKit.h>
#import <CoreServices/CoreServices.h>

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

static void add_editor_app(NSWorkspace *workspace, NSURL *appURL, NSString *iconCacheDirectory, NSMutableSet<NSString *> *seenBundleIdentifiers, NSMutableArray<NSDictionary *> *apps) {
    NSBundle *bundle = [NSBundle bundleWithURL:appURL];
    NSString *bundleIdentifier = [bundle bundleIdentifier];
    if (!bundle || !bundleIdentifier || [seenBundleIdentifiers containsObject:bundleIdentifier]) {
        return;
    }

    NSString *displayName = editor_app_display_name(bundle, appURL);
    [seenBundleIdentifiers addObject:bundleIdentifier];
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

NSString *scan_editor_apps_json(NSString *iconCacheDirectory) {
    NSArray<NSString *> *extensions = @[@"txt", @"md", @"swift", @"js", @"ts", @"tsx", @"jsx", @"json", @"py", @"rs", @"go", @"java", @"c", @"cpp", @"h", @"hpp", @"toml", @"yaml", @"yml"];
    NSWorkspace *workspace = [NSWorkspace sharedWorkspace];
    NSFileManager *fileManager = [NSFileManager defaultManager];
    NSMutableSet<NSString *> *seenBundleIdentifiers = [NSMutableSet set];
    NSMutableArray<NSDictionary *> *apps = [NSMutableArray array];
    NSString *probeDirectoryName = [@"portal-editor-probe-" stringByAppendingString:[[NSUUID UUID] UUIDString]];
    NSString *probeDirectory = [NSTemporaryDirectory() stringByAppendingPathComponent:probeDirectoryName];

    if (![fileManager createDirectoryAtPath:probeDirectory withIntermediateDirectories:YES attributes:nil error:nil]) {
        return @"[]";
    }

    NSURL *defaultFolderAppURL = [workspace URLForApplicationToOpenURL:[NSURL fileURLWithPath:probeDirectory isDirectory:YES]];
    if (defaultFolderAppURL) {
        add_editor_app(workspace, defaultFolderAppURL, iconCacheDirectory, seenBundleIdentifiers, apps);
    }

    for (NSString *ext in extensions) {
        NSString *probeName = [@"portal-editor-probe" stringByAppendingPathExtension:ext];
        NSString *probePath = [probeDirectory stringByAppendingPathComponent:probeName];
        [fileManager createFileAtPath:probePath contents:[NSData data] attributes:nil];
        NSURL *probeURL = [NSURL fileURLWithPath:probePath];
        CFArrayRef appURLs = LSCopyApplicationURLsForURL((CFURLRef)probeURL, kLSRolesEditor);
        if (!appURLs) {
            continue;
        }
        for (CFIndex index = 0; index < CFArrayGetCount(appURLs); index++) {
            NSURL *appURL = (NSURL *)CFArrayGetValueAtIndex(appURLs, index);
            add_editor_app(workspace, appURL, iconCacheDirectory, seenBundleIdentifiers, apps);
        }
        CFRelease(appURLs);
    }

    [fileManager removeItemAtPath:probeDirectory error:nil];

    NSData *jsonData = [NSJSONSerialization dataWithJSONObject:apps options:0 error:nil];
    if (!jsonData) {
        return @"[]";
    }

    NSString *json = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
    return [json autorelease];
}
