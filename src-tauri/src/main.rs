#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{
    command, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, Window,
    WindowBuilder, WindowEvent, WindowUrl,
};

#[command]
fn start_drag(window: Window) {
    let _ = window.start_dragging();
}

#[command]
fn minimize_window(window: Window) {
    window.minimize().unwrap();
}

#[command]
fn toggle_maximize_window(window: Window) {
    if window.is_maximized().unwrap() {
        window.unmaximize().unwrap();
    } else {
        window.maximize().unwrap();
    }
}

#[command]
fn close_window(window: Window) {
    window.hide().unwrap();
}

#[command]
fn open_devtools(window: Window) {
    window.open_devtools();
}

fn main() {
    let vencord_script = include_str!("../../Vencord.user.js");
    let vencord_styles = include_str!("../../vencordstyles");

    let polyfill = r#"
        // Bloqueia DevTools de todas as formas (Atalhos e Context Menu) - Executa em todos os frames
        document.addEventListener('contextmenu', e => e.preventDefault(), true);
        document.addEventListener('keydown', (e) => {
            const { ctrlKey, shiftKey, metaKey, key, keyCode } = e;
            
            // F12, Ctrl+Shift+I, Ctrl+Shift+J, Ctrl+Shift+C, Ctrl+U, Ctrl+S, Ctrl+P (Print), Ctrl+R (Reload)
            if (
                key === 'F12' || 
                keyCode === 123 ||
                ((ctrlKey || metaKey) && shiftKey && (key === 'I' || key === 'J' || key === 'C' || keyCode === 73 || keyCode === 74 || keyCode === 67)) ||
                ((ctrlKey || metaKey) && (key === 'u' || key === 'U' || key === 's' || key === 'S' || key === 'p' || key === 'P' || key === 'r' || key === 'R' || 
                 keyCode === 85 || keyCode === 83 || keyCode === 80 || keyCode === 82))
            ) {
                e.preventDefault();
                e.stopPropagation();
                
                // Abre o DevTools apenas com Ctrl+Shift+I
                if ((ctrlKey || metaKey) && shiftKey && (key === 'I' || keyCode === 73)) {
                    if (window.__TAURI__ && window.__TAURI__.invoke) {
                        window.__TAURI__.invoke('open_devtools');
                    }
                }
                return false;
            }
        }, true);

        // Só executa no frame principal para evitar lentidão extrema
        if (window.top === window.self) {
            console.log('Ventauri: Iniciando no frame principal...');

            // Injeta os estilos hardcoded
            const injectStyles = () => {
                function optimizeCSS(css) {
                    return css
                        // remove backdrop-filter
                        .replace(/backdrop-filter\s*:[^;]+;/gi, '')

                        // remove blur/filter pesados
                        .replace(/filter\s*:[^;]*blur\([^)]+\)[^;]*;/gi, '')

                        // remove transition all
                        .replace(/transition\s*:\s*all[^;]+;/gi, '')

                        // remove animações
                        .replace(/animation\s*:[^;]+;/gi, '')

                        // remove box-shadow pesado
                        .replace(/box-shadow\s*:[^;]+;/gi, 'box-shadow: none;');
                }
                const optimizedStyles = optimizeCSS(`VENCORD_STYLES_PLACEHOLDER`);
                if (document.head && !document.querySelector('vencord-root')) {
                    const stylesContainer = document.createElement('div');
                    stylesContainer.innerHTML = `VENCORD_STYLES_PLACEHOLDER`;
                    const styles = stylesContainer.firstElementChild;
                    if (styles) {
                        document.head.appendChild(styles);
                        console.log('Ventauri: Estilos hardcoded injetados com sucesso.');
                    }
                }
            };

            // Tenta injetar imediatamente e também quando o DOM estiver pronto
            injectStyles();
            document.addEventListener('DOMContentLoaded', injectStyles);

            // Polyfills rápidos
            if (typeof GM_xmlhttpRequest === "undefined") {
                window.GM_xmlhttpRequest = (details) => {
                    const { url, method = "GET", data, headers, onload, onerror } = details;
                    fetch(url, { method, body: data, headers })
                        .then(async (r) => {
                            const text = await r.text();
                            onload({ status: r.status, responseText: text, responseHeaders: r.headers, response: text });
                        }).catch(onerror);
                };
            }
            window.unsafeWindow = window;

            // Arraste de janela instantâneo (Event Listener precoce)
            let dragTimeout = null;

            document.addEventListener('mousedown', (e) => {
                if (e.button !== 0) return;

                const observer = new MutationObserver(() => {
                    const bar = document.querySelector('[class*="titleBar_"]');

                    if (bar) {
                        bar.style.webkitAppRegion = "drag";
                        console.log("Drag aplicado");

                        observer.disconnect();
                    }
                });

                observer.observe(document.body, {
                    childList: true,
                    subtree: true
                });

                const interactive = e.target.closest(`
                    button,
                    a,
                    input,
                    textarea,
                    select,
                    [role="button"],
                    [role="tab"],
                    [role="menuitem"],
                    nav *,
                    [class*="button"],
                    [class*="tab"],
                    [class*="link"]
                `);

                if (interactive) return;

                dragTimeout = setTimeout(async () => {
                    try {
                        await window.__TAURI__.invoke("start_drag");
                    } catch (err) {
                        console.error(err);
                    }
                }, 120); // tempo pequeno = sensação instantânea
            });

            document.addEventListener('mouseup', () => {
                if (dragTimeout) {
                    clearTimeout(dragTimeout);
                    dragTimeout = null;
                }
            });

            const xpath = '//*[@id="app-mount"]/div[2]/div/div[1]/div/div[2]/div/div/div/div[2]/div[1]/nav/ul/div/div/div[7]';
            
            const task = () => {
                // 1. Esconder elemento solicitado (XPath)
                try {
                    const res = document.evaluate(xpath, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                    const el = res.singleNodeValue;
                    if (el && el.style.display !== 'none') {
                        el.style.setProperty('display', 'none', 'important');
                        el.style.setProperty('height', '0', 'important');
                    }
                } catch(e){}

                // 2. Injetar branding "Ventauri" (Canto Esquerdo)
                // Procura pela div de título ou qualquer cabeçalho de seção
                const titleContainer = document.querySelector('[class*="title_c38106"]') || 
                                     document.querySelector('[class*="title_"]') ||
                                     document.querySelector('section[class*="title_"]');

                if (titleContainer && !document.querySelector('.ventauri-title')) {
                    const title = document.createElement('div');
                    title.className = 'ventauri-title';
                    title.textContent = 'Ventauri';
                    title.style.cssText = `
                        position: absolute;
                        left: 10px;
                        z-index: 1000;
                        font-family: "gg sans", "Noto Sans", sans-serif;
                        font-size: 15px !important;
                        font-weight: 850 !important;
                        color: #fff !important;
                        margin-right: 15px !important;
                        margin-left: 10px !important;
                        display: flex !important;
                        align-items: center !important;
                        flex-shrink: 0 !important;
                        letter-spacing: 0.5px !important;
                        text-transform: uppercase !important;
                        pointer-events: none !important;
                        white-space: nowrap !important;
                    `;
                    titleContainer.prepend(title);
                    titleContainer.style.setProperty('display', 'flex', 'important');
                    titleContainer.style.setProperty('align-items', 'center', 'important');
                }

                // 3. Injetar Botões (Canto Direito) e evitar sobreposição
                const bar = document.querySelector('[class*="titleBar_"]') || document.querySelector('[class*="typeWindows_"]') || document.querySelector('[class*="withFrame_"]') || document.querySelector('[class*="bar_c38106"]');
                if (bar) {
                    bar.style.position = 'relative';
                    
                    // Ajusta o padding da área de ícones original para dar espaço aos nossos botões
                    const trailingArea = bar.querySelector('[class*="trailing_"]') || 
                                         bar.querySelector('[class*="toolbar_"]') ||
                                         bar.querySelector('[class*="upperContainer_"]');
                    if (trailingArea) {
                        trailingArea.style.setProperty('margin-right', '100px', 'important');
                    }

                    if (!document.querySelector('.ventauri-controls')) {
                        const div = document.createElement('div');
                        div.className = 'ventauri-controls';
                        div.style.cssText = `
                            position: absolute !important;
                            right: 10px !important;
                            top: 50% !important;
                            transform: translateY(-50%) !important;
                            display: flex !important;
                            align-items: center !important;
                            gap: 10px !important;
                            z-index: 999999 !important;
                            -webkit-app-region: no-drag !important;
                        `;
                        
                        const createBtn = (color, cmd) => {
                            const b = document.createElement('button');
                            b.style.cssText = `
                                width: 12px !important;
                                height: 12px !important;
                                border-radius: 50% !important;
                                cursor: pointer !important;
                                border: none !important;
                                background: ${color} !important;
                                transition: filter 0.1s !important;
                                padding: 0 !important;
                                flex-shrink: 0 !important;
                            `;
                            b.onclick = async (e) => { 
                                e.preventDefault(); 
                                e.stopPropagation(); 
                                try {
                                    if (window.__TAURI__ && window.__TAURI__.invoke) {
                                        await window.__TAURI__.invoke(cmd);
                                    } else if (window.__TAURI__ && window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) {
                                        await window.__TAURI__.tauri.invoke(cmd);
                                    } else {
                                        console.error('Ventauri: window.__TAURI__.invoke not found');
                                    }
                                } catch(err) {
                                    console.error('Ventauri: Error invoking ' + cmd, err);
                                }
                            };
                            return b;
                        };

                        div.append(
                            createBtn('#ffbd2e', 'minimize_window'),
                            createBtn('#27c93f', 'toggle_maximize_window'),
                            createBtn('#ff5f56', 'close_window')
                        );
                        bar.appendChild(div);
                    }
                }
            };

            // Executa a cada 1s para manter a performance fluída
            setInterval(task, 1000);
            task();
        }
    "#;

    // Garante que tanto o polyfill quanto o Vencord só rodem no frame principal
    let full_script = format!(
        "if (window.top === window.self) {{ \n{}\n }}; \n if (window.top === window.self) {{ \n{}\n }};", 
        polyfill.replace("VENCORD_STYLES_PLACEHOLDER", vencord_styles), 
        vencord_script
    );

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Show Ventauri"))
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_maximize_window,
            close_window,
            open_devtools,
            start_drag
        ])
        .system_tray(system_tray)
        .on_system_tray_event(|app: &tauri::AppHandle, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        window.show().unwrap();
                        window.unminimize().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                _ => {}
            },
            SystemTrayEvent::LeftClick { .. } => {
                if let Some(window) = app.get_window("main") {
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
            }
            _ => {}
        })
        .setup(move |app| {
            let _window = WindowBuilder::new(
                app,
                "main",
                WindowUrl::External("https://discord.com/app".parse().unwrap()),
            )
            .title("Ventauri")
            .initialization_script(&full_script)
            .decorations(false)
            .transparent(true)
            .maximized(true)
            .min_inner_size(1024.0, 768.0)
            .build()?;

            Ok(())
        })
        .on_window_event(|event: tauri::GlobalWindowEvent| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
