#include <sys/types.h>
#include <grp.h>
#include <pwd.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int _su(char *user) {
	char *group, *end;

	uid_t uid = getuid();
	gid_t gid = getgid();

	group = strchr(user, ':');
	if (group)
		*group++ = '\0';

	struct passwd *pw = NULL;
	if (user[0] != '\0') {
		pw = getpwnam(user);
		uid_t nuid = strtol(user, &end, 10);
		if (*end == '\0')
			uid = nuid;
	}
	if (pw == NULL) {
		pw = getpwuid(uid);
	}
	if (pw != NULL) {
		uid = pw->pw_uid;
		gid = pw->pw_gid;
	}

	setenv("HOME", pw != NULL ? pw->pw_dir : "/", 1);

	if (group && group[0] != '\0') {
		/* group was specified, ignore grouplist for setgroups later */
		pw = NULL;

		struct group *gr = getgrnam(group);
		if (gr == NULL) {
			gid_t ngid = strtol(group, &end, 10);
			if (*end == '\0') {
				gr = getgrgid(ngid);
				if (gr == NULL)
					gid = ngid;
			}
		}
		if (gr != NULL)
			gid = gr->gr_gid;
	}

	if (pw == NULL) {
		if (setgroups(1, &gid) < 0) return -1;
	} else {
		int ngroups = 0;
		gid_t *glist = NULL;

		while (1) {
			int r = getgrouplist(pw->pw_name, gid, glist, &ngroups);
			if (r >= 0) {
				if (setgroups(ngroups, glist) < 0) return -1;
				break;
			}
			glist = realloc(glist, ngroups * sizeof(gid_t));
			if (glist == NULL) return -1;
		}
	}

	if (setgid(gid) < 0) return -1;
	if (setuid(uid) < 0) return -1;
	return 0;
}
