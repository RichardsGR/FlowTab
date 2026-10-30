pub struct TrayLabels {
    pub show_search: &'static str,
    pub manage_pages: &'static str,
    pub import_bookmarks: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
}

pub fn labels(lang: &str) -> TrayLabels {
    match lang {
        "zh-CN" => TrayLabels {
            show_search: "显示搜索框",
            manage_pages: "管理页面索引",
            import_bookmarks: "导入书签",
            settings: "设置",
            quit: "退出",
        },
        "zh-TW" => TrayLabels {
            show_search: "顯示搜尋框",
            manage_pages: "管理頁面索引",
            import_bookmarks: "匯入書籤",
            settings: "設定",
            quit: "結束",
        },
        "ja" => TrayLabels {
            show_search: "検索を表示",
            manage_pages: "ページを管理",
            import_bookmarks: "ブックマークを読み込む",
            settings: "設定",
            quit: "終了",
        },
        "ko" => TrayLabels {
            show_search: "검색 표시",
            manage_pages: "페이지 관리",
            import_bookmarks: "북마크 가져오기",
            settings: "설정",
            quit: "종료",
        },
        "es" => TrayLabels {
            show_search: "Mostrar búsqueda",
            manage_pages: "Gestionar páginas",
            import_bookmarks: "Importar marcadores",
            settings: "Ajustes",
            quit: "Salir",
        },
        "fr" => TrayLabels {
            show_search: "Afficher la recherche",
            manage_pages: "Gérer les pages",
            import_bookmarks: "Importer les favoris",
            settings: "Réglages",
            quit: "Quitter",
        },
        "de" => TrayLabels {
            show_search: "Suche anzeigen",
            manage_pages: "Seiten verwalten",
            import_bookmarks: "Lesezeichen importieren",
            settings: "Einstellungen",
            quit: "Beenden",
        },
        "pt" => TrayLabels {
            show_search: "Mostrar pesquisa",
            manage_pages: "Gerenciar páginas",
            import_bookmarks: "Importar favoritos",
            settings: "Configurações",
            quit: "Sair",
        },
        "ru" => TrayLabels {
            show_search: "Показать поиск",
            manage_pages: "Управление страницами",
            import_bookmarks: "Импортировать закладки",
            settings: "Настройки",
            quit: "Выход",
        },
        _ => TrayLabels {
            show_search: "Show Search",
            manage_pages: "Manage Pages",
            import_bookmarks: "Import Bookmarks",
            settings: "Settings",
            quit: "Quit",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_languages() {
        assert_eq!(labels("zh-CN").settings, "设置");
        assert_eq!(labels("zh-TW").settings, "設定");
        assert_eq!(labels("ja").quit, "終了");
        assert_eq!(labels("ko").settings, "설정");
        assert_eq!(labels("es").quit, "Salir");
        assert_eq!(labels("fr").settings, "Réglages");
        assert_eq!(labels("de").quit, "Beenden");
        assert_eq!(labels("pt").settings, "Configurações");
        assert_eq!(labels("ru").quit, "Выход");
    }

    #[test]
    fn falls_back_to_english() {
        let labels = labels("unknown");
        assert_eq!(labels.show_search, "Show Search");
        assert_eq!(labels.manage_pages, "Manage Pages");
        assert_eq!(labels.import_bookmarks, "Import Bookmarks");
        assert_eq!(labels.settings, "Settings");
        assert_eq!(labels.quit, "Quit");
    }
}
