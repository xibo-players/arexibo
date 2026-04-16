#include <QApplication>
#include <QIODevice>
#include <QBuffer>

#include "view.h"

Window::Window(QString base_uri, QScreen *screen, int inspect, callback cb, void *cb_ptr) :
    QMainWindow(),
    base_uri(base_uri),
    selected_screen(screen),
    cb(cb),
    cb_ptr(cb_ptr),
    layout_width(1920),
    layout_height(1080)
{
    setWindowFlags(windowFlags() | Qt::FramelessWindowHint);
    setWindowIcon(QIcon(":/assets/logo.png"));
    setStyleSheet("background-color: black;");

    view = new QWebEngineView(this);

    channel = new QWebChannel(this);
    view->page()->setWebChannel(channel);
    auto interface = new JSInterface(this);
    channel->registerObject("arexibo", interface);

    if (inspect) {
        auto devtools_window = new QMainWindow();
        auto devtools = new QWebEngineView();
        devtools_window->setWindowTitle("Arexibo - Inspector");
        devtools_window->setWindowIcon(QIcon(":/assets/logo.png"));
        devtools_window->setCentralWidget(devtools);
        devtools_window->resize(1000, 600);
        devtools_window->show();
        view->page()->setDevToolsPage(devtools->page());
    } else {
        QGuiApplication::setOverrideCursor(Qt::BlankCursor);        
    }

    connect(this, SIGNAL(navigateTo(QString)), this, SLOT(navigateToImpl(QString)));
    connect(this, SIGNAL(screenShot()), this, SLOT(screenShotImpl()));
    connect(this, SIGNAL(setTitle(QString)), this, SLOT(setWindowTitle(QString)));
    connect(this, SIGNAL(setSize(int, int, int, int)),
            this, SLOT(setSizeImpl(int, int, int, int)));
    connect(this, SIGNAL(runJavascript(QString)),
            this, SLOT(runJavascriptImpl(QString)));

    view->setUrl(QUrl(base_uri + "0.xlf.html"));
}

void Window::navigateToImpl(QString file) {
    view->setUrl(QUrl(base_uri + file));
}

void Window::screenShotImpl()
{
    QImage img(view->size(), QImage::Format_ARGB32);
    view->render(&img);
    QByteArray array;
    QBuffer buffer(&array);
    buffer.open(QIODevice::WriteOnly);
    img.save(&buffer, "PNG");
    cb(cb_ptr, CB_SCREENSHOT, (intptr_t)(const char *)array, array.size(), 0);
}

void Window::setSizeImpl(int pos_x, int pos_y, int size_x, int size_y)
{
    if (selected_screen)
        setScreen(selected_screen);
    QRect screenGeometry = screen()->geometry();
    int offset_x = screenGeometry.x();
    int offset_y = screenGeometry.y();
    int screen_w = screenGeometry.width();
    int screen_h = screenGeometry.height();

    // need to scale Xibo values (meant to be real pixels) by the device pixel ratio
    auto ratio = screen()->devicePixelRatio();
    pos_x = std::round(pos_x / ratio);
    pos_y = std::round(pos_y / ratio);
    size_x = std::round(size_x / ratio);
    size_y = std::round(size_y / ratio);

    if (size_x == 0) size_x = screen_w;
    if (size_y == 0) size_y = screen_h;

    // calculate window position and size
    if (size_x == screen_w && size_y == screen_h && pos_x == 0 && pos_y == 0) {
        resize(size_x, size_y);
        move(offset_x, offset_y);
        showFullScreen();
        std::cout << "INFO : [arexibo::qt] size: full screen ("
                  << size_x*ratio << "x" << size_y*ratio << ")" << std::endl;
    } else {
        setWindowState(windowState() & ~Qt::WindowFullScreen);
        resize(size_x, size_y);
        move(offset_x + pos_x, offset_y + pos_y);
        std::cout << "INFO : [arexibo::qt] size: windowed ("
                  << size_x*ratio << "x" << size_y*ratio << ")+"
                  << pos_x*ratio << "+" << pos_y*ratio << std::endl;
    }

    adjustScale(layout_width, layout_height);
}

void Window::adjustScale(int layout_w, int layout_h)
{
    layout_width = layout_w;
    layout_height = layout_h;

    // need to scale Xibo values (meant to be real pixels) by the device pixel ratio
    auto ratio = screen()->devicePixelRatio();
    layout_w = std::round(layout_w / ratio);
    layout_h = std::round(layout_h / ratio);

    int window_w = width();
    int window_h = height();

    if (window_w == 0 || window_h == 0 || layout_h == 0 || layout_w == 0)
        return;

    // the easy case: direct match
    if (window_w == layout_w && window_h == layout_h) {
        view->move(0, 0);
        view->resize(layout_w, layout_h);
        view->setZoomFactor(1.0);
        std::cout << "INFO : [arexibo::qt] scale: window = layout ("
                  << layout_w*ratio << "x" << layout_h*ratio << ")" << std::endl;
        return;
    }

    // adjust position of webview within the window, and apply the scale
    double window_aspect = (double)window_w / (double)window_h;
    double layout_aspect = (double)layout_w / (double)layout_h;
    double scale_factor;
    if (window_aspect > layout_aspect) {
        scale_factor = (double)window_h / (double)layout_h;
        int webview_w = (int)((double)layout_w * scale_factor);
        view->move((window_w - webview_w) / 2, 0);
        view->resize(webview_w, window_h);
        view->setZoomFactor(scale_factor);
    } else {
        scale_factor = (double)window_w / (double)layout_w;
        int webview_h = (int)((double)layout_h * scale_factor);
        view->move(0, (window_h - webview_h) / 2);
        view->resize(window_w, webview_h);
        view->setZoomFactor(scale_factor);
    }
    std::cout << "INFO : [arexibo::qt] scale: window ("
              << window_w*ratio << "x" << window_h*ratio << "), layout ("
              << layout_w*ratio << "x" << layout_h*ratio << "), result: ("
              << view->width()*ratio << "x" << view->height()*ratio << ")+"
              << view->x()*ratio << "+" << view->y()*ratio
              << " with zoom " << scale_factor << std::endl;
}

void Window::runJavascriptImpl(QString js)
{
    std::cout << "INFO : [arexibo::qt] run JavaScript: " << js.toStdString() << std::endl;
    view->page()->runJavaScript(js);
}

// Callbacks from JavaScript

void JSInterface::jsLayoutInit(int id, int width, int height)
{
    std::cout << "INFO : [arexibo::qt] layout " << id << " initialized" << std::endl;
    wnd->adjustScale(width, height);
    wnd->cb(wnd->cb_ptr, CB_LAYOUT_INIT, id, width, height);
}

void JSInterface::jsLayoutDone(int id)
{
    wnd->cb(wnd->cb_ptr, CB_LAYOUT_NEXT, id, 0, 0);
}

void JSInterface::jsLayoutPrev(int id)
{
    wnd->cb(wnd->cb_ptr, CB_LAYOUT_PREV, id, 0, 0);
}

void JSInterface::jsLayoutJump(int id, int which)
{
    wnd->cb(wnd->cb_ptr, CB_LAYOUT_JUMP, id, which, 0);
}

void JSInterface::jsCommand(QString code)
{
    std::string std_code = code.toStdString();
    wnd->cb(wnd->cb_ptr, CB_COMMAND, (intptr_t)std_code.c_str(), 0, 0);
}

void JSInterface::jsShell(QString command, int with_shell)
{
    std::string std_cmd = command.toStdString();
    wnd->cb(wnd->cb_ptr, CB_SHELL, (intptr_t)std_cmd.c_str(), with_shell, 0);
}

void JSInterface::jsStopShell(int kill_mode)
{
    wnd->cb(wnd->cb_ptr, CB_STOPSHELL, kill_mode, 0, 0);
}
