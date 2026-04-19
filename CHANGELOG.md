# Changelog

## 0.4.0 - Apr 2026

- Prevent path traversal in embedded webserver.
- Fix runtime segfaults with zeromq on some newer distributions.
- Allow giving "--screen" on command line.
- Properly handle high-dpi screens in Qt part.
- Allow giving "--debug" on command line, ignore CMS given loglevel.

## 0.3.3 - Mar 2026

- Allow "pdf" content type.
- Add desktop entry file and icon to repository.
- Fix problems with PDF display by using an embedded copy of
  the pdf.js library.

## 0.3.2 - May 2025

- Implement shell commands triggered by layouts.
- Allow "localvideo" content type.
- Set a minimum media duration of 1 second.
- Implement zindex property for layout regions.

## 0.3.1 - Apr 2025

- Implement player commands.
- Implement trigger actions/webhooks.
- Implement resource file purging as directed from CMS.
- Better debugging of SOAP responses.
- Parse embedded duration and numitems from resources.
- Properly translate references to view port size in content.
- Store raw XML responses from CMS for debugging.

## 0.3.0 - Mar 2025

- Switch to Qt6 for displaying web content. Arexibo now requires
  a C++ compiler, CMake and Qt including QtWebEngine to build.
- Add connection timeout for CMS connection.
- Automatically retranslate layouts on translator code update.
- Use a random port number for the internal web server.

## 0.2.8 - Feb 2025

- Add command line option to allow starting without a
  connection to the CMS, showing the last cached schedule.
- Fix display of background images.

## 0.2.7 - Jan 2025

- Update dependencies, require Rust 1.75.

## 0.2.6 - Jan 2025

- Add command line option to disabled HTTPS certificate
  verification when connecting to the CMS.
- Fix "stretch" scaling for images in layouts.

## 0.2.5 - Jul 2024

- Allow giving an initial display name on the command line,
  defaults to the hostname.
- Remove dependencies on jQuery in generated JavaScript.

## 0.2.4 - Jul 2024

- Update dependencies, require Rust 1.66.

## 0.2.3 - Jun 2022

- Fix broken unit test.

## 0.2.2 - May 2022

- Ensure media playback works from the internal web server
  by avoiding chunked responses.

## 0.2.1 - May 2022

- Fix some layout bugs.
- Remove official Xibo branding.

## 0.2.0 - Feb 2022

- Initial public release.
