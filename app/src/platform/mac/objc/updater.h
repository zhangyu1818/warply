#import <stdbool.h>

typedef void (*WarplySparkleEventCallback)(int, const char *, const char *);

void warply_sparkle_set_event_callback(WarplySparkleEventCallback);
bool warply_sparkle_start(void);
bool warply_sparkle_check_for_update_information(void);
bool warply_sparkle_check_for_updates(void);
