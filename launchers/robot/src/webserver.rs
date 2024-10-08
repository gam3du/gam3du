// static EXIT_FLAG: AtomicBool = AtomicBool::new(false);

// let webserver_thread = {
//     let event_sender = event_sender.clone();
//     let api = api.clone();
//     thread::spawn(move || {
//         debug!("thread[webserver]: starting server");
//         http_server(&event_sender, &api, &EXIT_FLAG);
//         debug!("thread[webserver]: exit");
//     })
// };

// debug!("thread[game loop]: instruct webserver to stop now");
// EXIT_FLAG.store(true, Ordering::Relaxed);

// debug!("Waiting for webserver to exit …");
// webserver_thread.join().unwrap();

// fn http_server(command_sender: &Sender<EngineEvent>, api: &Api, exit_flag: &'static AtomicBool) {
//     let server = Server::http("0.0.0.0:8000").unwrap();

//     'next_request: loop {
//         let request = match server.recv_timeout(Duration::from_millis(50)) {
//             Ok(Some(request)) => request,
//             Ok(None) => {
//                 if exit_flag.load(Ordering::Relaxed) {
//                     break 'next_request;
//                 }
//                 continue 'next_request;
//             }
//             Err(error) => {
//                 error!("{error}");
//                 break 'next_request;
//             }
//         };

//         let url = request.url();
//         let Some(url) = url.strip_prefix(&format!("/{}/", api.name)) else {
//             request
//                 .respond(Response::from_string("unknown api").with_status_code(404))
//                 .unwrap();
//             continue;
//         };

//         let command = Identifier(url.to_owned());

//         let response = Response::from_string(format!("{command:?}"));

//         // FIXME: Extract parameters from response
//         command_sender
//             .send(EngineEvent::RobotEvent {
//                 command: Identifier(url.to_owned()),
//                 parameters: vec![],
//             })
//             .unwrap();

//         request.respond(response).unwrap();
//     }
// }
