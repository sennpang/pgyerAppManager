# pgyerAppManager

USAGE:
    pgyer-uploader [FLAGS] [OPTIONS]

FLAGS:
        --check      check network
    -h, --help       Prints help information
    -l, --list       list my apps
    -V, --version    Prints version information

OPTIONS:
    -k, --key <STRING>                 Sets the api key
    -r, --remove <STRING>              app key that you want to delete
        --removeBuild <STRING>         build key that you want to delete
    -c, --channel <STRING>             build channel shortcut
    -d, --description <STRING>         build update description
    -f, --file <FILE>                  Sets the upload file
        --info <STRING>                get build info with build key
    -i, --installDate <NUMBER>         build install date, 1=buildInstallStartDate~buildInstallEndDate, 2=forever
    -e, --installEndDate <STRING>      build install start date, format: yyyy-MM-dd
    -s, --installStartDate <STRING>    build install end date, format: yyyy-MM-dd
    -t, --installType <NUMBER>         build install type, 1=public, 2=password, 3=invite
        --page <NUMBER>                page number
    -p, --password <STRING>            build password, required if installType=2
