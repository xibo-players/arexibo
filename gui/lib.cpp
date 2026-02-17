#include <QApplication>
#include <QMainWindow>
#include <QScreen>
#include <QtWebEngineCore/QWebEngineProfile>
#include <QtWebEngineCore/QWebEngineSettings>
#include <QtWebEngineWidgets/QWebEngineView>

#include "lib.h"
#include "view.h"

// For some reason, this constructor is not automatically called
int qInitResources_res();

QApplication *the_app = nullptr;
Window *the_wnd = nullptr;

int fake_argc = 1;
char *fake_argv[] = {(char *)"arexibo", nullptr};

void setup(const char *base_uri, const char *screen, int inspect, int debug,
           callback cb, void *cb_ptr) {
    if (the_wnd) return;

    if (debug)
        qputenv("QTWEBENGINE_CHROMIUM_FLAGS",
                "--single-process --enable-logging --log-level=0 --v=1");

    qInitResources_res();

    QCoreApplication::setOrganizationName("arexibo");
    the_app = new QApplication(fake_argc, fake_argv);

    auto screens = QApplication::screens();
    QScreen *selected_screen = nullptr;

    if (strcmp(screen, "list") == 0) {
        std::cout << "INFO : [arexibo::qt] listing screens:" << std::endl;
        int i = 1;
        foreach (auto scr, screens) {
            std::cout << "number " << i << " - name " << scr->name().toStdString() << std::endl;
            i++;
        }
    } else {
        int n = atoi(screen);
        if (n > 0 && n <= screens.length())
            selected_screen = screens[n - 1];
        else
            foreach (auto scr, screens)
                if (scr->name() == screen)
                    selected_screen = scr;
    }
    if (selected_screen)
        std::cout << "INFO : [arexibo::qt] selected screen: " <<
            selected_screen->name().toStdString() << std::endl;

    auto settings = QWebEngineProfile::defaultProfile()->settings();
    settings->setAttribute(QWebEngineSettings::ScreenCaptureEnabled, true);
    settings->setAttribute(QWebEngineSettings::PlaybackRequiresUserGesture, false);

    the_wnd = new Window(base_uri, selected_screen, inspect, cb, cb_ptr);
    the_wnd->show();
}

void run() {
    if (!the_app) return;
    the_app->exec();
}

void navigate(const char *file) {
    if (!the_wnd) return;
    emit the_wnd->navigateTo(file);
}

void screenshot() {
    if (!the_wnd) return;
    emit the_wnd->screenShot();
}

void set_title(const char *title) {
    if (!the_wnd) return;
    emit the_wnd->setTitle(title);
}

void set_size(int pos_x, int pos_y, int size_x, int size_y) {
    if (!the_wnd) return;
    emit the_wnd->setSize(pos_x, pos_y, size_x, size_y);
}

void run_js(const char *js) {
    if (!the_wnd) return;
    emit the_wnd->runJavascript(js);
}
