#include <QtGui/QApplication>
#include <QTranslator>
#include <QTextCodec>
#include <QtDebug>

#include "mainwindow.h"

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);

    QTextCodec::setCodecForCStrings(QTextCodec::codecForName("UTF-8"));
    QTextCodec::setCodecForLocale(QTextCodec::codecForName("UTF-8"));
    QTextCodec::setCodecForTr(QTextCodec::codecForName("UTF-8"));

    QTranslator *translator = new QTranslator;
    if (translator->load("/usr/share/svgcleaner/translations/svgcleaner"
                         +QLocale::system().name().remove(QRegExp("_.."))))
        qApp->installTranslator(translator);
    else
        qDebug()<<"Can't load translation.";

    MainWindow w;
    w.show();

    return a.exec();
}

