
use crate::MyWorld;
use cucumber::steps;

// Any type that implements cucumber::World + Default can be the world
steps!(MyWorld => {
        given regex "the following (.*) transports" |world, name, _step| {
            // Set up your context in given steps
            world.s = format!("{}", name[1]);
        };

        // when regex "^I choose a (.*) transport$" |world, _, _step| {
        //     // Take actions
        //     let new_string = format!("{}", &world.s);
        //     world.s = new_string;
        // };

        // then regex "^init the (.*) transport$" |world, name, _step| {
        //     // Check that the outcomes to be observed have occurred
        //     assert_eq!(world.s, name[1]);
        //     let mut t: Box<dyn Transport + 'static> = match name[1].as_str() {
        //         "internal" => Box::new(internal::TransportInternal::new()),
        //         "local" => Box::new(local::TransportLocal::new()),
        //         "network" => Box::new(network::TransportNetwork::new()),
        //         _ => panic!("Unknown transport type")
        //     };
        //     t.init().unwrap();
        //     assert_eq!(t.name(), name[1]);
        // };

        // given regex "^an inited (.*) transport$" |_world, name, _step| {
        //    let mut t: Box<dyn Transport + 'static> = match name[1].as_str() {
        //         "internal" => Box::new(internal::TransportInternal::new()),
        //         "local" => Box::new(local::TransportLocal::new()),
        //         "network" => Box::new(network::TransportNetwork::new()),
        //         _ => panic!("Unknown transport type")
        //     };
        //     t.init().unwrap();
        //     assert_eq!(t.name(), name[1]);
        //     assert_eq!(t.is_inited(), true);
        // };

        // then regex "init the (.*) message manager" |_world, name, _step| {
        //    let mut t: Box<dyn Transport + 'static> = match name[1].as_str() {
        //         "internal" => Box::new(internal::TransportInternal::new()),
        //         "local" => Box::new(local::TransportLocal::new()),
        //         "network" => Box::new(network::TransportNetwork::new()),
        //         _ => panic!("Unknown transport type")
        //     };
        //     t.init().unwrap();
        //     assert_eq!(t.name(), name[1]);
        //     assert_eq!(t.is_inited(), true);
        //     let mut mm = msgbus::msgmgr::MsgMgr::new(vec![(0..10, t)]);
        //     mm.init().unwrap();
        //     assert_eq!(mm.is_inited(), true);
        // };

        // given regex "an inited (.*) msgmgr" |_world, name, _step| {
        //    let mut t: Box<dyn Transport + 'static> = match name[1].as_str() {
        //         "internal" => Box::new(internal::TransportInternal::new()),
        //         "local" => Box::new(local::TransportLocal::new()),
        //         "network" => Box::new(network::TransportNetwork::new()),
        //         _ => panic!("Unknown transport type")
        //     };
        //     t.init().unwrap();
        //     assert_eq!(t.name(), name[1]);
        //     assert_eq!(t.is_inited(), true);
        //     let mut mm = msgbus::msgmgr::MsgMgr::new(vec![(0..10, t)]);
        //     mm.init().unwrap();
        //     assert_eq!(mm.is_inited(), true);
        // };

        // then regex "init the (.*) bus" |_world, name, _step| {
        //    let mut t: Box<dyn Transport + 'static> = match name[1].as_str() {
        //         "internal" => Box::new(internal::TransportInternal::new()),
        //         "local" => Box::new(local::TransportLocal::new()),
        //         "network" => Box::new(network::TransportNetwork::new()),
        //         _ => panic!("Unknown transport type")
        //     };
        //     t.init().unwrap();
        //     assert_eq!(t.name(), name[1]);
        //     assert_eq!(t.is_inited(), true);
        //     let mut mm = msgbus::msgmgr::MsgMgr::new(vec![(0..10, t)]);
        //     mm.init().unwrap();
        //     assert_eq!(mm.is_inited(), true);
        //     let mut r = rmb::Rmb::new(mm);
        //     r.init().unwrap();

        // };

        // then regex "querying the msgmgr should show (.*) transport" | _world, name, _step | {
        //    let mut t: Box<dyn Transport + 'static> = match name[1].as_str() {
        //         "internal" => Box::new(internal::TransportInternal::new()),
        //         "local" => Box::new(local::TransportLocal::new()),
        //         "network" => Box::new(network::TransportNetwork::new()),
        //         _ => panic!("Unknown transport type")
        //     };
        //     t.init().unwrap();
        //     assert_eq!(t.name(), name[1]);
        //     assert_eq!(t.is_inited(), true);
        //     let mut mm = msgbus::msgmgr::MsgMgr::new(vec![(0..10, t)]);
        //     mm.init().unwrap();
        //     assert_eq!(mm.is_inited(), true);
        //     let mut r = rmb::Rmb::new(mm);
        //     r.init().unwrap();
        //     assert_eq!(r.get_transport_names().unwrap()[0], name[1]);

        // };
    //     then regex r"^we can (.*) rules with regex$" |_world, matches, _step| {
    //         // And access them as an array
    //         assert_eq!(matches[1], "implement");
    //     };

    //     then regex r"^we can also match (\d+) (.+) types$" (usize, String) |_world, num, word, _step| {
    //         // `num` will be of type usize, `word` of type String
    //         assert_eq!(num, 42);
    //         assert_eq!(word, "olika");
    //     };

    //     then "we can use data tables to provide more parameters" |_world, step| {
    //         let table = step.table().unwrap().clone();

    //         assert_eq!(table.header, vec!["key", "value"]);

    //         let expected_keys = table.rows.iter().map(|row| row[0].to_owned()).collect::<Vec<_>>();
    //         let expected_values = table.rows.iter().map(|row| row[1].to_owned()).collect::<Vec<_>>();

    //         assert_eq!(expected_keys, vec!["a", "b"]);
    //         assert_eq!(expected_values, vec!["fizz", "buzz"]);
    //     };
    });
