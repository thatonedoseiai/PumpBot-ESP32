//! This module contains all the strings and their utf8 translations into various languages for 
//! PB. Each string is encoded as a struct that has nine fields, one for each language. A
//! `LanguageString` can be indexed using the `[]` operator, e.g.
//! `TEXT_LANGUAGE_NAME[Fr]` will return the name "français".

use core::ops::Index;

/// Represents a language that [LanguageString]s support.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Jp,
    Fr,
    Es,
    Pt,
    Zh,
    Cn,
    Ru,
    De
}

/// Represents a string that has translations in English, Japanese, French, Spanish, Portuguese,
/// Simplified and Traditional Chinese, Russian, and German.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LanguageString {
    en: &'static str,
    jp: &'static str,
    fr: &'static str,
    es: &'static str,
    pt: &'static str,
    zh: &'static str,
    cn: &'static str,
    ru: &'static str,
    de: &'static str,
}

impl LanguageString {
    pub const fn const_index(&self, lang: Lang) -> &&'static str {
        match lang {
            Lang::En => &self.en,
            Lang::Jp => &self.jp,
            Lang::Fr => &self.fr,
            Lang::Es => &self.es,
            Lang::Pt => &self.pt,
            Lang::Zh => &self.zh,
            Lang::Cn => &self.cn,
            Lang::Ru => &self.ru,
            Lang::De => &self.de,
        }
    }

    pub const fn const_string(s: &'static str) -> Self {
        LanguageString {
            en: s,
            jp: s,
            fr: s,
            es: s,
            pt: s,
            zh: s,
            cn: s,
            ru: s,
            de: s,
        }
    }
}

impl Index<Lang> for LanguageString {
    type Output = &'static str;

    fn index(&self, lang: Lang) -> &&'static str {
        self.const_index(lang)
    }
}

impl Into<u8> for Lang {
    fn into(self) -> u8 {
        match self {
            Lang::En => 0,
            Lang::Jp => 1,
            Lang::Fr => 2,
            Lang::Es => 3,
            Lang::Pt => 4,
            Lang::Zh => 5,
            Lang::Cn => 6,
            Lang::Ru => 7,
            Lang::De => 8,
        }
    }
}

impl From<u8> for Lang {
    fn from(val: u8) -> Self {
        match val {
            1 => Lang::Jp,
            2 => Lang::Fr,
            3 => Lang::Es,
            4 => Lang::Pt,
            5 => Lang::Zh,
            6 => Lang::Cn,
            7 => Lang::Ru,
            8 => Lang::De,
            _ => Lang::En,
        }
    }
}

pub const TEXT_LANGUAGE_NAME: LanguageString = LanguageString {
    en: "English",
    jp: "日本語",
    fr: "Français",
    es: "Español",
    pt: "Português",
    zh: "简体中文",
    cn: "繁體中文",
    ru: "Русский",
    de: "Deutsch",
};

pub const TEXT_WELCOME: LanguageString = LanguageString {
    en: "Welcome to",
    jp: "PumpBotへ",
    fr: "Bienvenue sur",
    es: "¡Bienvenido a",
    pt: "Bem-vinde ao",
    zh: "欢迎使用",
    cn: "歡迎使用",
    ru: "Добро подаловать",
    de: "Willkommen zu",
};


pub const TEXT_WELCOME_A: LanguageString = LanguageString {
    en: "PumpBot!",
    jp: "ようこそ!",
    fr: "PumpBot!",
    es: "PumpBot!",
    pt: "PumpBot!",
    zh: "PumpBot！",
    cn: "PumpBot！",
    ru: "в PumpBot!",
    de: "PumpBot!",
};

pub const TEXT_PRESSENC: LanguageString = LanguageString {
    en: "Press ⤓ to Continue",
    jp: "⤓を押して進む",
    fr: "Appuyez sur ⤓ pour continuer",
    es: "Presiona ⤓ para continuar",
    pt: "Pressione ⤓ para continuar",
    zh: "按⤓来开始",
    cn: "㩒⤓繼續",
    ru: "Нажмите ⤓ чтобы продолжить",
    de: "Drücke ⤓ Um Fortzufahren",
};

pub const TEXT_LANGUAGE: LanguageString = LanguageString {
    en: "Language",
    jp: "言語",
    fr: "Langue",
    es: "Idioma",
    pt: "Idioma",
    zh: "语言",
    cn: "語言",
    ru: "Язык",
    de: "Sprache",
};

pub const TEXT_SETTINGS: LanguageString = LanguageString {
    en: "Settings",
    jp: "設定",
    fr: "Options",
    es: "Configuración Avanzada",
    pt: "Opções",
    zh: "设定",
    cn: "設置",
    ru: "Настройки",
    de: "Einstellungen",
};

pub const TEXT_ADVANCED_SETTINGS: LanguageString = LanguageString {
    en: "Advanced",
    jp: "アドバンス設定",
    fr: "Options Avancées",
    es: "Configuración Avanzada",
    pt: "Opções Avançadas",
    zh: "高级设置",
    cn: "詳細",
    ru: "Расширенные настройки",
    de: "Erweiterte Einstellungen",
};

pub const TEXT_OK: LanguageString = LanguageString {
    en: "OK",
    jp: "OK",
    fr: "OK",
    es: "OK",
    pt: "OK",
    zh: "好",
    cn: "好",
    ru: "ОК",
    de: "OK",
};

pub const TEXT_BACK: LanguageString = LanguageString {
    en: "Back",
    jp: "戻る",
    fr: "Retour",
    es: "Atrás",
    pt: "Voltar",
    zh: "返回",
    cn: "返回",
    ru: "Назаԭ",
    de: "Zurück",
};

pub const TEXT_YES: LanguageString = LanguageString {
    en: "Yes",
    jp: "はい",
    fr: "Oui",
    es: "Si",
    pt: "Sim",
    zh: "是",
    cn: "是",
    ru: "Ԭа",
    de: "Ja",
};

pub const TEXT_NO: LanguageString = LanguageString {
    en: "No",
    jp: "いいえ",
    fr: "Non",
    es: "No",
    pt: "Não",
    zh: "否",
    cn: "否",
    ru: "Нет",
    de: "Nein",
};

pub const TEXT_CANCEL: LanguageString = LanguageString {
    en: "Cancel",
    jp: "キャンセル",
    fr: "Annuler",
    es: "Cancelar",
    pt: "Cancelar",
    zh: "取消",
    cn: "取消",
    ru: "Отмена",
    de: "Abbrechen",
};

pub const TEXT_NEXT: LanguageString = LanguageString {
    en: "Next",
    jp: "次へ",
    fr: "Suivant",
    es: "Siᴳuiente",
    pt: "Próximo",
    zh: "继续",
    cn: "繼續",
    ru: "Слеԭԩюԝее",
    de: "Nächstes",
};

pub const TEXT_CHOOSE_LANG: LanguageString = LanguageString {
    en: "Choose your Language",
    jp: "言語を選ぶ",
    fr: "Choisissez votre Langue",
    es: "Elige tu idioma",
    pt: "Selecione seu idioma",
    zh: "选择语言",
    cn: "選擇語言",
    ru: "Выберите свой язык",
    de: "Wähle deine Sprache",
};

pub const TEXT_SETUP_PB: LanguageString = LanguageString {
    en: "How would you like to",
    jp: "PumpBotをどうやって",
    fr: "Comment voulez-vous",
    es: "¿Cómo te gustaría",
    pt: "Como gostaria de configurar",
    zh: "你要怎么样",
    cn: "你要怎麽樣",
    ru: "Как бы вы хотели",
    de: "Wie würdest du PumpBot",
};

pub const TEXT_SETUP_PB_A: LanguageString = LanguageString {
    en: "setup PumpBot?",
    jp: "設定したいですか？",
    fr: "configurer Pumpbot?",
    es: "configurar PumpBot?",
    pt: " o PumpBot?",
    zh: "设置PumpBot?",
    cn: "設置PumpBot?",
    ru: "настроить PumpBot?",
    de: "gerne aufsetzen?",
};

pub const TEXT_WIFI_SETUP: LanguageString = LanguageString {
    en: "Wi-Fi Setup",
    jp: "Wi-Fiでセットアップ",
    fr: "Options Wi-Fi",
    es: "Configuración de Wi-Fi",
    pt: "Configuração de Wi-Fi",
    zh: "网上设置",
    cn: "網上設定",
    ru: "Настройки Wi-Fi",
    de: "Wi-Fi Setup",
};

pub const TEXT_TOOLTIP_WIFI_SETUP: LanguageString = LanguageString {
    en: "Set up PumpBot by connecting",
    jp: "他のデバイスに",
    fr: "Configurer Pumpbot en connectant",
    es: "Configura PumpBot conectando",
    pt: "Configurar PumpBot conectando",
    zh: "连接其他台机来",
    cn: "連接第二部機來",
    ru: "Настройте PumpBot подключив",
    de: "Setze PumpBot durch das Verbinden",
};

pub const TEXT_TOOLTIP_WIFI_SETUP_A: LanguageString = LanguageString {
    en: "another device",
    jp: "接続して設定する",
    fr: "un autre appareil",
    es: "otro dispositivo",
    pt: "a um dispositivo",
    zh: "设置Pumpbot",
    cn: "設置PumpBot",
    ru: "другое устройство",
    de: "eines anderen Gerätes auf",
};

pub const TEXT_CONNECT_HERE: LanguageString = LanguageString {
    en: "Connect to this Wi-Fi network",
    jp: "PumpBotを設定するためにこの",
    fr: "Connectez-vous à ce Wi-Fi",
    es: "Conecta a esta red Wi-Fi",
    pt: "Conecte a essa rede de Wi-Fi ",
    zh: "连接这个网路",
    cn: "連接這個網絡",
    ru: "Подключитесь к этой сети Wi-Fi,",
    de: "Verbinde zu diesem Wi-Fi Netzwerk um",
};

pub const TEXT_CONNECT_HERE_A: LanguageString = LanguageString {
    en: "to configure PumpBot",
    jp: "ネットワークに接続してください",
    fr: "pour configurer Pumpbot",
    es: "para configurar PumpBot",
    pt: "para configurar PumpBot",
    zh: "来设置PumpBot",
    cn: "來設置PumpBot",
    ru: "чтобы настроить PumpBot",
    de: "PumpBot zu konfigurieren",
};

pub const TEXT_GO_TO_URL: LanguageString = LanguageString {
    en: "Open this URL in your web browser",
    jp: "このリンクをブラウザーで開く",
    fr: "Ouvrez ce lien dans votre navigateur",
    es: "Abre este link en tu navegador",
    pt: "Abra essa página no seu",
    zh: "用浏览器开这条链接",
    cn: "用瀏覽器開這條鏈接",
    ru: "Откройте этот URL-адрес",
    de: "Öffne diese URL in ",
};

pub const TEXT_GO_TO_URL_A: LanguageString = LanguageString {
    en: "",
    jp: "",
    fr: "",
    es: "",
    pt: "navegador de internet",
    zh: "",
    cn: "",
    ru: "в своем веб-браузере",
    de: "deinem Web-Browser",
};

pub const TEXT_WIFI_STARTING: LanguageString = LanguageString {
    en: "Starting Wi-Fi Network...",
    jp: "Wi-Fi発散中",
    fr: "Lancement du réseau Wi-Fi...",
    es: "Iniciando Red Wi-Fi...",
    pt: "Inicializando rede de Wi-Fi...",
    zh: "Wi-Fi开启中",
    cn: "Wi-Fi開啟中",
    ru: "Запускаем сеть Wi-Fi...",
    de: "Starte Wi-Fi Netzwerk...",
};

pub const TEXT_WIFI_STOP: LanguageString = LanguageString {
    en: "Stop Wi-Fi Setup",
    jp: "Wi-Fi設定を停止して",
    fr: "Arrêter la configuration Wi-Fi",
    es: "Detener la configuración Wi-Fi",
    pt: "Parar configuração de Wi-Fi",
    zh: "停止Wi-Fi设置",
    cn: "停止Wi-Fi設置",
    ru: "Остановить настройку Wi-Fi",
    de: "Stoppe Wi-Fi Setup",
};

pub const TEXT_SAVE_EXIT: LanguageString = LanguageString {
    en: "Save and Exit",
    jp: "保存して終了する",
    fr: "Sauvegarder et quitter",
    es: "Guardar y Salir",
    pt: "Salvar e Sair",
    zh: "保存并完成",
    cn: "保存而完了",
    ru: "Сохранить и выйти",
    de: "Speichern und Beenden",
};

pub const TEXT_WIFI_CONTINUE_WITHOUT: LanguageString = LanguageString {
    en: "Continue without",
    jp: "Wi-Fiに接続せずに",
    fr: "Continuer sans",
    es: "¿Continuar sin",
    pt: "Continuar sem conectar",
    zh: "不连接网路",
    cn: "不連接網絡",
    ru: "Продолжить без",
    de: "Fortfahren ohne mit",
};

pub const TEXT_WIFI_CONTINUE_WITHOUT_A: LanguageString = LanguageString {
    en: "connecting to Wi-Fi?",
    jp: "続きますか？",
    fr: "se connecter au Wi-Fi? ",
    es: "conexión Wi-Fi?",
    pt: "a uma rede Wi-Fi?",
    zh: "这样设置?",
    cn: "怎樣設定?",
    ru: "подключения к Wi-Fi?",
    de: "Wi-Fi zu verbinden?",
};

pub const TEXT_SETUP_COMPLETE: LanguageString = LanguageString {
    en: "Setup Complete!",
    jp: "セットアップ完了！",
    fr: "Configuration terminée!",
    es: "¡Configuración completa!",
    pt: "Configuração finalizada!",
    zh: "设置完了",
    cn: "設置完了",
    ru: "Установка завершена!",
    de: "Setup beendet!",
};

pub const TEXT_STANDALONE_SETUP: LanguageString = LanguageString {
    en: "Standalone Setup",
    jp: "デバイスでセットアップ",
    fr: "Configuration autonome",
    es: "Configuración independiente",
    pt: "Configuração Independente",
    zh: "独立设置",
    cn: "獨立設定",
    ru: "Автономная установка",
    de: "Unabhängiges Setup",
};

pub const TEXT_TOOLTIP_STANDALONE_SETUP: LanguageString = LanguageString {
    en: "Set up PumpBot without connecting ",
    jp: "他のデバイスに",
    fr: "Configurez Pumpbot sans connecter ",
    es: "Configurar PumpBot sin conectar",
    pt: "Configurar PumpBot não conectando",
    zh: "不要连接其他台机来",
    cn: "不要連接其他部機來",
    ru: "Настройте PumpBot без подключения",
    de: "Setze PumpBot ohne zusätzliche",
};

pub const TEXT_TOOLTIP_STANDALONE_SETUP_A: LanguageString = LanguageString {
    en: "another device",
    jp: "接続せずに設定する",
    fr: "un autre appareil",
    es: "otro dispositivo",
    pt: "a um dispositivo",
    zh: "设置PumpBot",
    cn: "設定PumpBot",
    ru: "другого устройства",
    de: "Geräteverbindung auf",
};

pub const TEXT_WIFI_SETTINGS: LanguageString = LanguageString {
    en: "Wi-Fi Settings",
    jp: "Wi-Fi設定",
    fr: "Paramètres Wi-Fi",
    es: "Ajustes del Wi-Fi",
    pt: "Opções de Wi-Fi",
    zh: "Wi-Fi设定",
    cn: "Wi-Fi設置",
    ru: "Настройки Wi-Fi",
    de: "Wi-Fi Einstellungen",
};

pub const TEXT_TOOLTIP_WIFI_CONNECT: LanguageString = LanguageString {
    en: "Connect and Disconnect to Wi-Fi Networks",
    jp: "Wi-Fiネットワークに接続と切断",
    fr: "Se connecter et se déconnecter au Wi-Fi",
    es: "Conectarse y desconectarse de redes Wi-Fi",
    pt: "Conectar e desconectar a redes Wi-Fi",
    zh: "Wi-Fi连接和断开",
    cn: "Wi-Fi連接和斷開",
    ru: "Подключение и отключение от сети Wi-Fi",
    de: "Verbinde und Trenne dich zu Wi-Fi Netzwerken",
};

pub const TEXT_NETWORK: LanguageString = LanguageString {
    en: "Network",
    jp: "ネットワーク",
    fr: "Réseau",
    es: "Red",
    pt: "Rede",
    zh: "网路",
    cn: "網絡",
    ru: "Сеть",
    de: "Netzwerk",
};

pub const TEXT_NETWORK_NAME: LanguageString = LanguageString {
    en: "Network Name",
    jp: "ネットワーク名",
    fr: "Nom du Réseau",
    es: "Nombre de Red",
    pt: "Nome de Rede",
    zh: "网路名",
    cn: "網絡名",
    ru: "Имя сети",
    de: "Netzwerkname",
};

pub const TEXT_PASSWORD: LanguageString = LanguageString {
    en: "Password",
    jp: "パスワード",
    fr: "Mot de passe",
    es: "Contraseña",
    pt: "Senha",
    zh: "网路密码",
    cn: "網絡密碼",
    ru: "Пароль",
    de: "Passwort",
};

pub const TEXT_IP_ADDR: LanguageString = LanguageString {
    en: "IP Address",
    jp: "IPアドレス",
    fr: "Adresse IP",
    es: "Dirección de IP",
    pt: "Endereço de IP",
    zh: "IP地址",
    cn: "IP地址",
    ru: "IP-адрес",
    de: "IP Adresse",
};

pub const TEXT_SEARCHING: LanguageString = LanguageString {
    en: "Searching...",
    jp: "検索中...",
    fr: "Recherche...",
    es: "Buscando...",
    pt: "Procurando...",
    zh: "搜查中",
    cn: "搜查中",
    ru: "Идёт поиск...",
    de: "Suche...",
};

pub const TEXT_SEARCH: LanguageString = LanguageString {
    en: "Search",
    jp: "検索",
    fr: "Rechercher",
    es: "Buscar",
    pt: "Procurar",
    zh: "搜查",
    cn: "搜查",
    ru: "Поиск",
    de: "Suche",
};

pub const TEXT_CANT_CONNECT: LanguageString = LanguageString {
    en: "Can't Connect",
    jp: "接続できません",
    fr: "Impossible de se connecter",
    es: "No se pudo conectar",
    pt: "Impossível Conectar",
    zh: "不可连线",
    cn: "不可連線",
    ru: "Невозможно подключиться",
    de: "Verbindung nicht möglich!",
};

pub const TEXT_CONNECT: LanguageString = LanguageString {
    en: "Connect",
    jp: "接続",
    fr: "Se connecter",
    es: "Conectar",
    pt: "Conectar",
    zh: "连线",
    cn: "連線",
    ru: "Подключить",
    de: "Verbinden",
};

pub const TEXT_DISCONNECT: LanguageString = LanguageString {
    en: "Disconnect",
    jp: "切断",
    fr: "Se déconnecter",
    es: "Desconectar",
    pt: "Disconectar",
    zh: "断开",
    cn: "斷開",
    ru: "Отключить",
    de: "Trennen",
};

pub const TEXT_DISCONNECTED: LanguageString = LanguageString {
    en: "Disconnected",
    jp: "切断されました",
    fr: "Déconnecté",
    es: "Desconectando",
    pt: "Disconectado",
    zh: "断开了",
    cn: "斷開了",
    ru: "Отключено",
    de: "Getrennt",
};

pub const TEXT_CONNECTING: LanguageString = LanguageString {
    en: "Connecting...",
    jp: "接続処理中…",
    fr: "Connexion...",
    es: "Conectando...",
    pt: "Conectando...",
    zh: "连线中",
    cn: "連線中...",
    ru: "Подключение...",
    de: "Verbinden...",
};

pub const TEXT_CONNECTED: LanguageString = LanguageString {
    en: "Connected!",
    jp: "接続済み！",
    fr: "Connecté!",
    es: "¡Conectado!",
    pt: "Conectado!",
    zh: "连线开了",
    cn: "連線完了",
    ru: "Подключено!",
    de: "Verbunden!",
};

pub const TEXT_CONNECTION_FAIL: LanguageString = LanguageString {
    en: "Connection Failed!",
    jp: "接続失敗！",
    fr: "Connexion échouée!",
    es: "¡Conexión fallida!",
    pt: "Conexão Falhou!",
    zh: "连线断了",
    cn: "連接斷了",
    ru: "Подключение не удалось!",
    de: "Verbindung fehlgeschlagen",
};

pub const TEXT_SERVER_SETTINGS: LanguageString = LanguageString {
    en: "Server Settings",
    jp: "サーバー設定",
    fr: "Paramètres Serveur",
    es: "Ajustes del Servidor",
    pt: "Opções de Servidor",
    zh: "服务器设定",
    cn: "服務器設置",
    ru: "Настройки сервера",
    de: "Server Einstellungen",
};

pub const TEXT_SERVER_ADDR: LanguageString = LanguageString {
    en: "Server Address",
    jp: "サーバーアドレス",
    fr: "Adresse du Serveur",
    es: "Dirección del Servidor",
    pt: "Endereço de Servidor",
    zh: "服务器地址",
    cn: "服務器地置",
    ru: "Адрес сервера",
    de: "Server Adresse",
};

pub const TEXT_SERVER_PASSWORD: LanguageString = LanguageString {
    en: "Server Password",
    jp: "サーバーパスワード",
    fr: "Mot de passe du serveur",
    es: "Contraseña del Servidor",
    pt: "Senha de Servidor",
    zh: "服务器密码",
    cn: "服務器密碼",
    ru: "Пароль сервера",
    de: "Server Passwort",
};

pub const TEXT_DISPLAY_SETTING: LanguageString = LanguageString {
    en: "Display Settings",
    jp: "画面設定",
    fr: "Paramètres d'affichage",
    es: "Configuración de pantalla",
    pt: "Opções de Exibição",
    zh: "画面设定",
    cn: "畫面設置",
    ru: "Настройки экрана",
    de: "Anzeigeeinstellungen",
};

pub const TEXT_TOOLTIP_DISPLAY_SETTING: LanguageString = LanguageString {
    en: "Change Display Brightness\nand Theme",
    jp: "画面の明るさとテーマを\n変えます",
    fr: "Changer le thème et\nla luminosité",
    es: "Cambiar el brillo y el\ntema de la pantalla",
    pt: "Mudar Brilho e Tema",
    zh: "改画面光度和颜色模式",
    cn: "改畫面光度和顏色模式",
    ru: "Изменить яркость и\nтему дисплея",
    de: "Bildschirmhelligkeit\nund Theme anpassen",
};

pub const TEXT_BRIGHTNESS: LanguageString = LanguageString {
    en: "Brightness",
    jp: "明るさ",
    fr: "Luminosité",
    es: "Brillo",
    pt: "Brilho",
    zh: "光度",
    cn: "光度",
    ru: "Яркость",
    de: "Helligkeit",
};

pub const TEXT_THEME: LanguageString = LanguageString {
    en: "Theme",
    jp: "テーマ:",
    fr: "Thème",
    es: "Tema",
    pt: "Tema",
    zh: "颜色模式",
    cn: "顏色模式",
    ru: "Тема",
    de: "Theme",
};

pub const TEXT_LIGHT_MODE: LanguageString = LanguageString {
    en: "Light Mode",
    jp: "ライトモード",
    fr: "Mode clair",
    es: "Modo Claro",
    pt: "Modo Claro",
    zh: "浅颜色模式",
    cn: "淺色模式",
    ru: "Светлый режим",
    de: "Heller Modus",
};

pub const TEXT_DARK_MODE: LanguageString = LanguageString {
    en: "Dark Mode",
    jp: "ダークモード",
    fr: "Mode sombre",
    es: "Modo Oscuro",
    pt: "Modo Escuro",
    zh: "深颜色模式",
    cn: "深色模式",
    ru: "Тёмный режим",
    de: "Dunkler Modus",
};

pub const TEXT_CUSTOM: LanguageString = LanguageString {
    en: "Custom",
    jp: "カスタム",
    fr: "Personnalisé",
    es: "Personalizar",
    pt: "Personalizado",
    zh: "自定",
    cn: "自定義",
    ru: "кастомная",
    de: "Benutzerdefiniert",
};

pub const TEXT_CUSTOM_COLOR: LanguageString = LanguageString {
    en: "Custom Color",
    jp: "カスタムな色",
    fr: "Couleur personnalisé",
    es: "Personalizar Color",
    pt: "Cor Personalizada",
    zh: "自定颜色",
    cn: "自定義顏色",
    ru: "кастомная цвет",
    de: "Benutzerdefinierte Farbe",
};

pub const TEXT_RED: LanguageString = LanguageString {
    en: "Red",
    jp: "赤",
    fr: "Rouge",
    es: "Rojo",
    pt: "Vermelho",
    zh: "红",
    cn: "紅",
    ru: "Красный",
    de: "Rot",
};

pub const TEXT_GREEN: LanguageString = LanguageString {
    en: "Green",
    jp: "緑",
    fr: "Vert",
    es: "Verde",
    pt: "Verde",
    zh: "录",
    cn: "綠",
    ru: "Зелёный",
    de: "Grün",
};

pub const TEXT_BLUE: LanguageString = LanguageString {
    en: "Blue",
    jp: "青",
    fr: "Bleu",
    es: "Azul",
    pt: "Azul",
    zh: "蓝",
    cn: "藍",
    ru: "Синий",
    de: "Blau",
};

pub const TEXT_RGB_SETTINGS: LanguageString = LanguageString {
    en: "RGB Settings",
    jp: "RGB設定",
    fr: "Paramètres RVB",
    es: "Ajustes RGB",
    pt: "Opções RGB",
    zh: "RGB设定",
    cn: "RGB設置",
    ru: "Параметры РГБ",
    de: "RGB Einstellungen",
};

pub const TEXT_TOOLTIP_RGB_SETTINGS: LanguageString = LanguageString {
    en: "Change the RGB brightness, color, mode, and more",
    jp: "RGBの明るさや色やモードを変えます",
    fr: "Changer la luminosité RVB, la couleur, le mode, ...",
    es: "Cambie el brillo, el color, el modo y más del RGB",
    pt: "Mudar brilho, cor, modo de RGB",
    zh: "改RGB光度、颜色、模式、和其他",
    cn: "改RGB光度、顏色、模式、而其他",
    ru: "Изменить настройки РГБ (освещение, цвет, режим и т.д.)",
    de: "Ändere RGB Helligkeit, Farbe, Modus und mehr",
};

pub const TEXT_RGB_BRIGHTNESS: LanguageString = LanguageString {
    en: "RGB Brightness",
    jp: "RGBの明るさ",
    fr: "Luminosité RVB",
    es: "Brillo del RGB",
    pt: "Brilho de RGB",
    zh: "RGB光度",
    cn: "RGB光度",
    ru: "Яркость РГБ",
    de: "RGB Helligkeit",
};

pub const TEXT_RGB_SPEED: LanguageString = LanguageString {
    en: "RGB Speed",
    jp: "RGBの速さ",
    fr: "Vitesse RVB",
    es: "Velocidad del RGB",
    pt: "Velocidade de RGB",
    zh: "RGB速度",
    cn: "RGB速度",
    ru: "Скорость РГБ",
    de: "RGB Geschwindigkeit",
};

pub const TEXT_RGB_MODE: LanguageString = LanguageString {
    en: "RGB Mode",
    jp: "RGBモード",
    fr: "Mode RVB",
    es: "Modo del RGB",
    pt: "Modo de RGB",
    zh: "RGB模式",
    cn: "RGB模式",
    ru: "Режим РГБ",
    de: "RGB Modus",
};

pub const TEXT_RAINBOW: LanguageString = LanguageString {
    en: "Rainbow",
    jp: "虹色",
    fr: "Arc-en-ciel",
    es: "Arcoíris",
    pt: "Arco-Íris",
    zh: "彩虹色",
    cn: "彩虹色",
    ru: "Радуга",
    de: "Regenbogen",
};

pub const TEXT_FADE: LanguageString = LanguageString {
    en: "Fade ",
    jp: "変る色",
    fr: "Fondu",
    es: "Desteñir",
    pt: "Gradiente",
    zh: "渐变颜色",
    cn: "週期顏色",
    ru: "Тусклость",
    de: "Fade",
};

pub const TEXT_SOLID: LanguageString = LanguageString {
    en: "Solid Color",
    jp: "変らない色",
    fr: "Couleur unie",
    es: "Color Sólido",
    pt: "Cor Sólida",
    zh: "不变颜色",
    cn: "不變顏色",
    ru: "Сплошной",
    de: "Feste Farbe",
};

pub const TEXT_RGB_OFF: LanguageString = LanguageString {
    en: "Off",
    jp: "色なし",
    fr: "Éteint",
    es: "Apagar RGB",
    pt: "Desligado",
    zh: "离开",
    cn: "熄滅",
    ru: "Выключить",
    de: "Aus",
};

pub const TEXT_COLOR: LanguageString = LanguageString {
    en: "Color",
    jp: "色",
    fr: "Couleur",
    es: "Color",
    pt: "Cor",
    zh: "颜色",
    cn: "顏色",
    ru: "Цвет",
    de: "Farbe",
};

pub const TEXT_COLOR_1: LanguageString = LanguageString {
    en: "Color 1",
    jp: "第一の色",
    fr: "Couleur 1",
    es: "Color 1",
    pt: "Cor 1",
    zh: "第一颜色",
    cn: "第一個顏色",
    ru: "Цвет 1",
    de: "Farbe 1",
};

pub const TEXT_COLOR_2: LanguageString = LanguageString {
    en: "Color 2",
    jp: "第二の色",
    fr: "Couleur 2",
    es: "Color 2",
    pt: "Cor 2",
    zh: "第二颜色",
    cn: "第二個顏色",
    ru: "Цвет 2",
    de: "Farbe 2",
};

pub const TEXT_COLOR_PICKER: LanguageString = LanguageString {
    en: "Color Picker",
    jp: "色選択",
    fr: "Palette de couleur",
    es: "Selector de Color",
    pt: "Selecionador de Cor",
    zh: "选色器",
    cn: "任意顏色",
    ru: "Выбор цвета",
    de: "Farbauswahl",
};

pub const TEXT_ADDON_SETTINGS: LanguageString = LanguageString {
    en: "Add-on Settings",
    jp: "アドオン設定",
    fr: "Options supplémentaires",
    es: "Configuración de complementos",
    pt: "Opções de Add-ons",
    zh: "插件设定",
    cn: "插件設定",
    ru: "Настройка Периферий",
    de: "Erweiterungseinstellungen ",
};

pub const TEXT_MPRLS: LanguageString = LanguageString {
    en: "Unit of Pressure",
    jp: "空気圧の単位",
    fr: "Unité de pression",
    es: "Unidad de Presión",
    pt: "Unidade de Pressão",
    zh: "气压单位",
    cn: "氣壓單位",
    ru: "Единицы измерения",
    de: "Druckeinheit",
};

pub const TEXT_PRESSURE: LanguageString = LanguageString {
    en: "Pressure",
    jp: "空圧",
    fr: "Pression",
    es: "Presión",
    pt: "Pressão",
    zh: "气压",
    cn: "氣壓",
    ru: "Давление",
    de: "Druck",
};

pub const TEXT_OUTPUT_SETTINGS: LanguageString = LanguageString {
    en: "Output Settings",
    jp: "出力設定",
    fr: "Paramètres de sortie",
    es: "Ajustes de Salida",
    pt: "Opções de Saída de Energia",
    zh: "输出设定",
    cn: "出力設置",
    ru: "Настройки вывода",
    de: "Ausgabeeinstellungen",
};

pub const TEXT_OUTPUT_MODE: LanguageString = LanguageString {
    en: "Output Mode",
    jp: "出力モード",
    fr: "Mode de sortie",
    es: "Modo de Salida",
    pt: "Modo de Saída de Energia",
    zh: "输出模式",
    cn: "出力模式",
    ru: "Режим вывода",
    de: "Ausgabemodus",
};

pub const TEXT_CHANNEL_NUM: LanguageString = LanguageString {
    en: "Channel",
    jp: "出力チャンネル",
    fr: "Port",
    es: "Canal",
    pt: "Canal",
    zh: "通道",
    cn: "出力通道",
    ru: "Канал",
    de: "Kanal",
};

pub const TEXT_PWM: LanguageString = LanguageString {
    en: "PWM",
    jp: "PWM",
    fr: "MLI",
    es: "PWM",
    pt: "PWM",
    zh: "脉宽调制",
    cn: "PWM",
    ru: "ШИМ",
    de: "PWM",
};

pub const TEXT_PWM_FREQ: LanguageString = LanguageString {
    en: "PWM Frequency",
    jp: "PWM周波数",
    fr: "Fréquence MLI",
    es: "Frecuencia del PWM",
    pt: "Frequência de PWM",
    zh: "调制频率",
    cn: "PWM頻",
    ru: "Частота ШИМ",
    de: "PWM Frequenz",
};

pub const TEXT_ANALOG_DIGITAL: LanguageString = LanguageString {
    en: "ON/OFF Mode",
    jp: "ON/OFFモード",
    fr: "Mode ON/OFF",
    es: "Modo Encendido/Apagado",
    pt: "Modo Ligado/Desligado",
    zh: "模拟/数字模式",
    cn: "模擬/數字模式",
    ru: "Режим ВКЛ/ВЫКЛ",
    de: "AN/AUS Modus",
};

pub const TEXT_ANALOG: LanguageString = LanguageString {
    en: "Analog",
    jp: "アナログ",
    fr: "Analogique",
    es: "Análogo",
    pt: "Analógico",
    zh: "模拟",
    cn: "模擬",
    ru: "Аналоговый",
    de: "Analog",
};

pub const TEXT_DIGITAL: LanguageString = LanguageString {
    en: "Digital",
    jp: "デジタル",
    fr: "Numérique",
    es: "Digital",
    pt: "Digital",
    zh: "数字",
    cn: "數字",
    ru: "Цифровой",
    de: "Digital",
};

pub const TEXT_PWM_WIZARD: LanguageString = LanguageString {
    en: "PWM Wizard",
    jp: "PWMウィザード",
    fr: "Assistant de configuration MLI",
    es: "Asistente de PWM",
    pt: "Assistente de PWM",
    zh: "脉宽调制向导",
    cn: "PWM設定嚮導",
    ru: "ШИМ мастер",
    de: "PWM Assistent",
};

pub const TEXT_MAP_PWM_RANGE: LanguageString = LanguageString {
    en: "Map PWM Range",
    jp: "PWMの強さを変る",
    fr: "Configurer la plage MLI",
    es: "Mapa de Rango del PWM",
    pt: "Mapear largura de PWM",
    zh: "映射调制限制",
    cn: "PWM範圍",
    ru: "Преобразовать промежуток ШИМ-а",
    de: "Plane PWM Reichweite",
};

pub const TEXT_LOWEST_VALUE: LanguageString = LanguageString {
    en: "Lowest Value",
    jp: "最低強さ",
    fr: "Valeur minimale",
    es: "Valor Mínimo",
    pt: "Valor mais baixo",
    zh: "最低的限度",
    cn: "最低的限度",
    ru: "Мин. значение",
    de: "Niedrigster Wert",
};

pub const TEXT_HIGHEST_VALUE: LanguageString = LanguageString {
    en: "Highest Value",
    jp: "最高強さ",
    fr: "Valeur maximale",
    es: "Valor Máximo",
    pt: "Valor mais alto",
    zh: "最高的限度",
    cn: "最高的限度",
    ru: "Макс. значение",
    de: "Höchster Wert",
};

pub const TEXT_USB_CONNECTED: LanguageString = LanguageString {
    en: "USB Connected!",
    jp: "USB接続済み",
    fr: "USB connecté!",
    es: "¡USB Conectado!",
    pt: "USB conectado!",
    zh: "USB连接了",
    cn: "USB連接了",
    ru: "USB Подключён!",
    de: "USB verbunden!",
};

pub const TEXT_USB_DISCONNECTED: LanguageString = LanguageString {
    en: "USB Disconnected!",
    jp: "USB切断済み",
    fr: "USB déconnecté!",
    es: "¡USB Desconectado!",
    pt: "USB disconectado!",
    zh: "USB断开了",
    cn: "USB斷開了",
    ru: "USB Отключён!",
    de: "USB getrennt!",
};

pub const TEXT_SETTINGS_DISPLAY: LanguageString = LanguageString {
    en: "Display",
    jp: "画面",
    fr: "Affichage",
    es: "Pantalla",
    pt: "Tela",
    zh: "画面",
    cn: "畫面",
    ru: "Монитор",
    de: "Anzeige",
};

pub const TEXT_SETTINGS_NETWORK: LanguageString = LanguageString {
    en: "Network",
    jp: "ネットワーク",
    fr: "Réseau",
    es: "Red",
    pt: "Rede",
    zh: "网路",
    cn: "網絡",
    ru: "Сеть",
    de: "Netzwerk",
};

pub const TEXT_SETTINGS_OUTPUT: LanguageString = LanguageString {
    en: "Output",
    jp: "出力",
    fr: "Sortie",
    es: "Salida",
    pt: "Saída",
    zh: "出力",
    cn: "出力",
    ru: "Выход",
    de: "Ausgabe",
};

pub const TEXT_SETTINGS_RGB: LanguageString = LanguageString {
    en: "RGB",
    jp: "RGB",
    fr: "RVB",
    es: "RGB",
    pt: "RGB",
    zh: "RGB",
    cn: "RGB",
    ru: "РГБ",
    de: "RGB",
};

pub const TEXT_SETTINGS_ADD_ONS: LanguageString = LanguageString {
    en: "Add-Ons",
    jp: "アドオン",
    fr: "Options supplémentaires",
    es: "Complementos",
    pt: "Add-Ons",
    zh: "插件",
    cn: "插件",
    ru: "Периферийное устройство",
    de: "Erweiterungen",
};

pub const TEXT_SETTINGS_APPS: LanguageString = LanguageString {
    en: "Apps",
    jp: "アプリ",
    fr: "Applications",
    es: "Aplicaciones",
    pt: "Apps",
    zh: "应用",
    cn: "應用",
    ru: "Программы",
    de: "Programme",
};

pub const TEXT_SETTINGS_DEVELOPER: LanguageString = LanguageString {
    en: "Developer",
    jp: "発展者設定",
    fr: "Développeur",
    es: "Desarrollador",
    pt: "Desenvolvedor",
    zh: "开发人设定",
    cn: "發展設定",
    ru: "Разработчик",
    de: "Entwickler",
};

pub const TEXT_SETTINGS_EXTRAS: LanguageString = LanguageString {
    en: "Extras",
    jp: "余分",
    fr: "Suppléments",
    es: "Extras",
    pt: "Adicional",
    zh: "附加东西",
    cn: "附加",
    ru: "дополнительные",
    de: "Extras",
};

pub const TEXT_SETTINGS_CREDITS: LanguageString = LanguageString {
    en: "Credits",
    jp: "作成者",
    fr: "Crédits",
    es: "Créditos",
    pt: "créditos",
    zh: "创造者",
    cn: "製造者",
    ru: "кредиты",
    de: "Credits",
};

pub const TEXT_SETTINGS_SOFTWARE: LanguageString = LanguageString {
    en: "Software",
    jp: "ソフト",
    fr: "Software",
    es: "Software",
    pt: "Programas",
    zh: "软件",
    cn: "軟體",
    ru: "программное",
    de: "Software",
};

pub const TEXT_SETTINGS_HARDWARE: LanguageString = LanguageString {
    en: "Hardware",
    jp: "ハードウェア",
    fr: "Carte",
    es: "Hardware",
    pt: "Hardware",
    zh: "电路板",
    cn: "電路板",
    ru: "аппаратное",
    de: "Hardware",
};

pub const TEXT_SETTINGS_TRANSLATIONS: LanguageString = LanguageString {
    en: "Translations",
    jp: "翻訳者",
    fr: "Traduction",
    es: "Traducción",
    pt: "Tradução",
    zh: "翻译",
    cn: "翻譯",
    ru: "перевод",
    de: "Übersetzung",
};

pub const TEXT_SETTINGS_CONCEPT: LanguageString = LanguageString {
    en: "Concept",
    jp: "アイディア",
    fr: "Idée",
    es: "Concepto",
    pt: "Conceito",
    zh: "概念",
    cn: "概念",
    ru: "концепция",
    de: "Konzept",
};

pub const TEXT_APPLICATIONS: LanguageString = LanguageString {
    en: "Applications",
    jp: "アプリ",
    fr: "Applications",
    es: "Aplicaciones",
    pt: "Aplicações",
    zh: "应用",
    cn: "應用",
    ru: "Приложения",
    de: "Programme",
};

pub const TEXT_APP_DOWNLOAD: LanguageString = LanguageString {
    en: "Download",
    jp: "アプリ読み込み",
    fr: "Télécharger",
    es: "Descarᴳar",
    pt: "Download",
    zh: "下载应用",
    cn: "下載應用",
    ru: "Скачать",
    de: "Herunterladen",
};

pub const TEXT_APP_DOWNLOADING: LanguageString = LanguageString {
    en: "Downloading...",
    jp: "読み込み中...",
    fr: "Téléchargement...",
    es: "Descargando...",
    pt: "Fazendo download...",
    zh: "下载中",
    cn: "下載中",
    ru: "Загрузка",
    de: "Lädt herunter...",
};

pub const TEXT_APP_DL_FROM_URL: LanguageString = LanguageString {
    en: "Download from URL",
    jp: "リンクから読み込み",
    fr: "Télécharger par lien",
    es: "Descargar desde URL",
    pt: "Fazer download por URL",
    zh: "去链接来下载",
    cn: "URL來下載",
    ru: "Скачать с URL",
    de: "Herunterladen von URL",
};

pub const TEXT_APP_URL: LanguageString = LanguageString {
    en: "URL",
    jp: "ﾘﾝｸ",
    fr: "Lien",
    es: "URL",
    pt: "URL",
    zh: "链接",
    cn: "URL",
    ru: "URL",
    de: "URL",
};

pub const TEXT_APP_GO: LanguageString = LanguageString {
    en: "Go!",
    jp: "行う",
    fr: "Go!",
    es: "ǏVamos!",
    pt: "Iniciar!",
    zh: "行动",
    cn: "行動",
    ru: "Впеԗёԭ!",
    de: "Los!",
};

pub const TEXT_APP_RUN: LanguageString = LanguageString {
    en: "Run App",
    jp: "開く",
    fr: "Lancer l'application",
    es: "Ejecutar Aplicación",
    pt: "Rodar app",
    zh: "开",
    cn: "開",
    ru: "Запустить приложение",
    de: "Programm ausführen",
};

pub const TEXT_APP_DEL: LanguageString = LanguageString {
    en: "Delete App",
    jp: "削除",
    fr: "Désinstaller l'application",
    es: "Eliminar Aplicación",
    pt: "Deletar app",
    zh: "删除",
    cn: "刪除",
    ru: "Удалить приложение",
    de: "Programm löschen",
};

pub const TEXT_APP_DEFAULT: LanguageString = LanguageString {
    en: "Set as Default",
    jp: "デフォルトになる",
    fr: "Définir par défaut",
    es: "Establecer por defecto",
    pt: "Definir como padrão",
    zh: "设为默认",
    cn: "變做缺省",
    ru: "Установить по умолчанию",
    de: "Als Standard setzen",
};

pub const TEXT_APP_MANAGE: LanguageString = LanguageString {
    en: "Manage App",
    jp: "アプリ管理",
    fr: "Paramétrer l'application",
    es: "Administrar Aplicación",
    pt: "Gerenciar app",
    zh: "应用管理",
    cn: "應用管理",
    ru: "Управление приложением",
    de: "Programm verwalten",
};

pub const TEXT_ENTER_TEXT: LanguageString = LanguageString {
    en: "Enter Text",
    jp: "テキスト入力",
    fr: "Entrer du Texte",
    es: "Ingresar Texto",
    pt: "Inserir texto",
    zh: "打字",
    cn: "打字",
    ru: "Введите текст",
    de: "Text eingeben",
};

pub const TEXT_PORT: LanguageString = LanguageString {
    en: "Port",
    jp: "ボート",
    fr: "Port",
    es: "Puerto",
    pt: "Porta",
    zh: "通讯埠",
    cn: "通訊埠",
    ru: "Порт",
    de: "Port",
};

pub const TEXT_NETWORK_SETTINGS: LanguageString = LanguageString {
    en: "Network Settings",
    jp: "ネットワーク設定",
    fr: "Paramètres réseau",
    es: "Configuración de Red",
    pt: "Opções de rede",
    zh: "网路设定",
    cn: "網絡設定",
    ru: "Настройки сети",
    de: "Netztwerkeinstellungen",
};

pub const TEXT_RESET: LanguageString = LanguageString {
    en: "Reset",
    jp: "リセット",
    fr: "Réinitialiser",
    es: "Reiniciar",
    pt: "Restaurar",
    zh: "重置",
    cn: "重置",
    ru: "Сбросить",
    de: "Zurücksetzen",
};

pub const TEXT_FACTORY_RESET: LanguageString = LanguageString {
    en: "Factory Reset",
    jp: "再セットアップ",
    fr: "Rétablir la configuration d'usine",
    es: "Reinicio de Fábrica",
    pt: "Restaurar padrões de fábrica",
    zh: "恢复出厂设置",
    cn: "回復出廠設定",
    ru: "Сбросить до заводских настроек",
    de: "Zurücksetzen auf Werksseinstellungen",
};

// TODO: get this localized
pub const TEXT_REFRESH: LanguageString = LanguageString {
    en: "Refresh",
    jp: "リロード",
    fr: "Recharger",
    es: "Recargar",
    pt: "Recarregar",
    zh: "再找网路",
    cn: "再搵網絡",
    ru: "обновить сети",
    de: "Refresh",
};

pub const TEXT_VOLTAGE: LanguageString = LanguageString {
    en: "Voltage",
    jp: "電圧",
    fr: "Tension électrique",
    es: "Voltaje",
    pt: "Tensão elétrica",
    zh: "電壓",
    cn: "電壓",
    ru: "Электри́ческое напряже́ние",
    de: "Spannung",
};

pub const TEXT_MIN: LanguageString = LanguageString {
    en: "Miniumum",
    jp: "最低",
    fr: "Minimum",
    es: "Mínimo",
    pt: "Mínimo",
    zh: "最低",
    cn: "最低",
    ru: "Минимум",
    de: "Minimum",
};

pub const TEXT_MAX: LanguageString = LanguageString {
    en: "Maximum",
    jp: "最高",
    fr: "Maximum",
    es: "Máximo",
    pt: "Máximo",
    zh: "最高",
    cn: "最高",
    ru: "Максимум",
    de: "Maximum",
};

pub const TEXT_SOLENOID_MODE: LanguageString = LanguageString {
    en: "Solenoid Mode",
    jp: "a",
    fr: "a",
    es: "a",
    pt: "a",
    zh: "a",
    cn: "a",
    ru: "a",
    de: "a",
};
