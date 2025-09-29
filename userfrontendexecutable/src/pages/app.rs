
use std::path::PathBuf;

use dioxus::prelude::*;

use crate::backend;
use crate::pages::*;
use crate::pages::colony_container_page::ColonyContainerPage;
use crate::pages::local_container_page::LocalContainerPage;
use crate::pages::general_container_page::GeneralContainerPage;
use crate::JobId;


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plugin {}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    InstallationPage,
    EntryPage,
    LocalStartPage(PathBuf),
    ColonyStartPage(PathBuf),
    //GenericStartPage,
    //LocalRunningPage(PathBuf),
    ContainerSelfConfiguratorPage(PathBuf, String),
    GenericObserverPage(JobId),
    ProjectOverviewPage,
    LabBookPage,
    //HTMLPage(String),
    PluginPage(Plugin),
    IFrame(String),
    TestPage
}


#[component]
pub fn CompleteApp() -> Element {
    // background threads executed independently, and connected by sender/receiver
    let (frontcomm, backcomm) = backend::commchannel_pair();
    //start background thread
    std::thread::spawn( || {
        backend::listen(backcomm);
    });

    // single thread communication (on frontend)
    let app_state = use_signal(|| AppState::InstallationPage as AppState);
    let senderhandle = use_signal(move || frontcomm);

//    let mut yrn_font_file = std::fs::File::open(YARNDINGS_FONT).expect("Unable to open font file asset");
//    let mut yrn_font = String::new();
//    yrn_font_file.read_to_string(&mut yrn_font).expect("Unable to read font file asset");
//
//    let mut inter_font_file = std::fs::File::open(INTER_FONT).expect("Unable to open font file asset");
//    let mut inter_font = String::new();
//    inter_font_file.read_to_string(&mut inter_font).expect("Unable to read font file asset");

    rsx! {
//        style { {inter_font.clone()} }
        //        style { {yrn_font.clone()} }
        style { {include_str!("./../../public/styles/norm.css")}}
        style { {include_str!("./../../public/styles/resets.css")}}

        style { {include_str!("./../../public/styles/installation_page.css")}}
        style { {include_str!("./../../public/styles/entry_page.css")}}
        style { {include_str!("./../../public/styles/local_container_page.css")}}
        style { {include_str!("./../../public/styles/colony_container_page.css")}}
        style { {include_str!("./../../public/styles/general_container_page.css")}}
        style { {include_str!("./../../public/styles/html_wrapper.css")}}
        style { {include_str!("./../../public/styles/iframe.css")}}
        style { {include_str!("./../../public/styles/remote_dir_test.css")}}
        style { {include_str!("./../../public/styles/project_overview_page.css")}}
        style { {include_str!("./../../public/styles/project_page.css")}}

        style { {include_str!("./../../public/styles/utilities.css")}}

        {
            #[cfg(debug_assertions)]
            rsx! {style { {include_str!("./../../public/styles/debug.css")} }}
        }
        {
            #[cfg(feature = "clrHue")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/hue_coloring.css")} }}
        }
        {
            #[cfg(feature = "clrArctic")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/Arctic.css")} }}
        }
        {
            #[cfg(feature = "clrAwink")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/Awink.css")} }}
        }
        {
            #[cfg(feature = "clrDistinction")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/Distinction.css")} }}
        }
        {
            #[cfg(feature = "clrIMI")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/IMI.css")} }}
        }
        {
            #[cfg(feature = "clrIMI-inv")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/IMI_inverted.css")} }}
        }
        {
            #[cfg(feature = "clrJebsen")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/Jebsen.css")} }}
        }
        {
            #[cfg(feature = "clrProud")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/ProudAndTorn.css")} }}
        }
        {
            #[cfg(feature = "clrSciam")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/Sciampagna.css")} }}
        }
        {
            #[cfg(feature = "clrSlumber")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/Slumber.css")} }}
        }
        {
            #[cfg(feature = "clrLight")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/LightTheme.css")} }}
        }
        {
            #[cfg(feature = "clrIMGs")]
            rsx! {style { {include_str!("./../../public/styles/color_schemes/img_backgrounds.css")} }}
        }

        div {
            match app_state() {
                AppState::InstallationPage => rsx! { InstallPage {app_state, comm_with_backend: senderhandle} },
                //AppState::EntryPage => rsx! { EntryPage {app_state, comm_with_backend: senderhandle} },
                //redirected for now
                AppState::EntryPage => rsx! { GeneralContainerPage {app_state, comm_with_backend: senderhandle} },
                AppState::LocalStartPage(_container_path) => rsx! { LocalContainerPage {app_state, comm_with_backend: senderhandle} },
                AppState::ColonyStartPage(_container_path) => rsx! { ColonyContainerPage {app_state, comm_with_backend: senderhandle}  },
                //AppState::LocalRunningPage(_container_path) => rsx! { ColonyContainerPage {app_state, comm_with_backend: senderhandle} },
                AppState::ContainerSelfConfiguratorPage(ptb, ip_address) => rsx! {ContainerSelfConfiguratorPage {app_state, comm_with_backend: senderhandle, title: "".to_string(), container: ptb, address: ip_address}},
                AppState::GenericObserverPage(job_id) => rsx! { GenericObserverPage {app_state, comm_with_backend: senderhandle, jobid: job_id} },
                //AppState::HTMLPage(_) => rsx! { HandleBarsTemplatePage {app_state, comm_with_backend: senderhandle} },
                AppState::PluginPage(_plugin) => rsx! { EntryPage {app_state, comm_with_backend: senderhandle} },
                AppState::IFrame(url) => rsx! { IFramePage { app_state, title: "Pending".to_owned(), source: url } },
                AppState::TestPage => rsx! { RemoteDirTestPage { app_state, comm_with_backend: senderhandle } },
                AppState::ProjectOverviewPage => rsx! { ProjectOverviewPage { app_state, comm_with_backend: senderhandle } },
                AppState::LabBookPage => rsx! { LabBookPage { app_state, comm_with_backend: senderhandle } },
            }
        }
    }
}








